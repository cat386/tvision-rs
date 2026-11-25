import contextlib
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import sqlite3
import tempfile
import threading
import unittest
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("harness", Path(__file__).with_name("harness.py"))
h = importlib.util.module_from_spec(spec)
spec.loader.exec_module(h)


def request(root, **extra):
    return dict(config={"base_url": "https://example.invalid/v1", "model": "test", "system": "test", "key_env": "DASHSCOPE_API_KEY"},
                key="fixture-api-key", workspace=str(root), journal=str(root/"runs.sqlite"), messages=[{"role": "user", "content": "task"}],
                conversation=0, **extra)


class HarnessTests(unittest.TestCase):
    def test_workspace_boundary_and_concurrent_file_change(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            runtime = h.Runtime(request(root))
            self.addCleanup(runtime.db.close)
            with self.assertRaises(ValueError):
                runtime.path("../outside")
            path = root/"test.txt"
            path.write_text("old", encoding="utf-8")
            with patch.object(runtime, "approve", return_value=True):
                read = runtime.execute("files_read", {"path": "test.txt"})
                path.write_text("external", encoding="utf-8")
                with self.assertRaises(ValueError):
                    runtime.execute("files_write", {"path": "test.txt", "content": "agent", "expected_sha256": read["sha256"]})
                self.assertEqual(path.read_text(), "external")
                runtime.execute("files_write", {"path": "new.txt", "content": "Привет", "expected_sha256": "new"})
            self.assertEqual((root/"new.txt").read_text(encoding="utf-8"), "Привет")
            with self.assertRaises(ValueError):
                runtime.execute("files_read", {"path": "new.txt", "count": -1})
            runtime.db.close()

    def test_public_urls_reject_localhost(self):
        for address in ["http://127.0.0.1/", "http://localhost/", "file:///etc/passwd", "http://169.254.169.254/"]:
            with self.assertRaises(ValueError):
                h.public_url(address)

    def test_real_http_tool_cycle_approvals_and_journal(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root/"source.txt").write_text("original", encoding="utf-8")
            calls = []
            replies = [
                {"role": "assistant", "content": None, "tool_calls": [{
                    "id": "read1", "type": "function", "function": {"name": "files_read", "arguments": '{"path":"source.txt"}'}}]},
                {"role": "assistant", "content": None, "tool_calls": [{
                    "id": "write1", "type": "function", "function": {"name": "files_write", "arguments": json.dumps({
                        "path": "answer.txt", "content": "готово", "expected_sha256": "new"})}}]},
                {"role": "assistant", "content": "Файл сохранён."},
            ]

            class Handler(BaseHTTPRequestHandler):
                def do_POST(self):
                    self.server.owner.assertEqual(self.path, "/v1/chat/completions")
                    self.server.owner.assertEqual(self.headers["Authorization"], "Bearer fixture-api-key")
                    body = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
                    calls.append(body)
                    response = json.dumps({"choices": [{"message": replies.pop(0)}]}).encode()
                    self.send_response(200)
                    self.send_header("Content-Length", str(len(response)))
                    self.end_headers()
                    self.wfile.write(response)

                def log_message(self, *args):
                    pass

            server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
            server.owner = self
            thread = threading.Thread(target=server.serve_forever, daemon=True)
            thread.start()
            runtime_request = request(root)
            runtime_request["config"]["base_url"] = f"http://127.0.0.1:{server.server_port}/v1"
            runtime = h.Runtime(runtime_request)
            events = []

            def approve(name, arguments):
                # Намерение уже сохранено до подтверждения и до изменения файла.
                row = runtime.db.execute("SELECT state FROM calls WHERE name=?", (name,)).fetchone()
                self.assertEqual(row[0], "pending")
                self.assertFalse((root/"answer.txt").exists())
                return True

            try:
                with patch.object(runtime, "approve", side_effect=approve), patch.object(h, "emit", side_effect=lambda kind, **data: events.append((kind, data))):
                    runtime.run()
            finally:
                server.shutdown()
                server.server_close()
                thread.join(timeout=3)
            self.assertEqual((root/"answer.txt").read_text(encoding="utf-8"), "готово")
            self.assertEqual(len(calls), 3)
            self.assertEqual(calls[1]["messages"][-1]["role"], "tool")
            self.assertEqual(calls[1]["messages"][-1]["tool_call_id"], "read1")
            self.assertEqual(events[-1], ("done", {"text": "Файл сохранён."}))
            with contextlib.closing(sqlite3.connect(runtime_request["journal"])) as db:
                self.assertEqual(db.execute("SELECT state FROM runs").fetchone()[0], "complete")
                self.assertEqual(db.execute("SELECT count(*) FROM calls WHERE state='complete'").fetchone()[0], 2)
                rows = str(db.execute("SELECT * FROM runs").fetchall())
                self.assertNotIn("fixture-api-key", rows)

    def test_denied_action_never_mutates_file(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            runtime = h.Runtime(request(root))
            try:
                with patch.object(runtime, "approve", return_value=False):
                    result = runtime.execute("files_write", {"path": "denied.txt", "content": "x", "expected_sha256": "new"})
                self.assertIn("отклонил", result)
                self.assertFalse((root/"denied.txt").exists())
            finally:
                runtime.db.close()

    def test_resume_pending_action_reads_state_instead_of_replaying(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            previous = h.Runtime(request(root))
            messages = [{"role": "system", "content": "system"}, {"role": "user", "content": "task"},
                        {"role": "assistant", "content": None, "tool_calls": [{
                            "id": "pending1", "type": "function", "function": {"name": "files_write", "arguments": "{}"}}]}]
            previous.checkpoint("running", messages)
            previous.db.execute("INSERT INTO calls VALUES(?,?,?,?,?,?)", (previous.run_id, "pending1", "files_write", "{}", "pending", None))
            previous.db.commit()
            previous.db.close()
            resumed = h.Runtime(request(root, resume=True))
            received = []
            def model(messages):
                received.extend(messages)
                return {"role": "assistant", "content": "Сначала проверим состояние."}
            with patch.object(resumed, "model", side_effect=model), patch.object(resumed, "execute") as execute, contextlib.redirect_stdout(io.StringIO()):
                resumed.run()
                execute.assert_not_called()
            tool_result = next(m for m in received if m["role"] == "tool")
            self.assertIn("Результат неизвестен", tool_result["content"])

    def test_terminal_timeout_and_output(self):
        import sys
        code, output = h.process_output([sys.executable, "-c", "print('ok')"], Path.cwd())
        self.assertEqual(code, 0)
        self.assertIn("ok", output)
        with self.assertRaises(TimeoutError):
            h.process_output([sys.executable, "-c", "import time; time.sleep(10)"], Path.cwd(), timeout=0.2)


if __name__ == "__main__":
    unittest.main()
