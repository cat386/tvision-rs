//! Местное время без внешних зависимостей.
//!
//! Unix — `localtime_r` из libc, Windows — `SystemTimeToTzSpecificLocalTime`.
//! Если перевести не удалось, возвращается время UTC.

use std::time::{SystemTime, UNIX_EPOCH};

/// Дата и время: год, месяц (1–12), день, час, минута, секунда.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateTime {
    pub year: i64,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}

/// Время UTC по числу секунд от 1970 года (алгоритм Говарда Хиннанта).
pub fn utc_from_secs(secs: i64) -> DateTime {
    let days = secs.div_euclid(86400);
    let rem = secs.rem_euclid(86400);
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = if month <= 2 { y + 1 } else { y };
    DateTime { year, month, day, hour: (rem / 3600) as u32, minute: ((rem % 3600) / 60) as u32, second: (rem % 60) as u32 }
}

fn secs_of(t: SystemTime) -> i64 {
    match t.duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_secs() as i64,
        Err(e) => -(e.duration().as_secs() as i64),
    }
}

/// Местное время для момента `t`.
pub fn local(t: SystemTime) -> DateTime {
    let secs = secs_of(t);
    imp::local(secs).unwrap_or_else(|| utc_from_secs(secs))
}

/// Текущее местное время.
pub fn now() -> DateTime {
    local(SystemTime::now())
}

#[cfg(unix)]
mod imp {
    use super::DateTime;
    use std::os::raw::{c_int, c_long};

    /// `struct tm`: первые 9 полей одинаковы везде; хвост (tm_gmtoff, tm_zone)
    /// закрываем запасом байт.
    #[repr(C)]
    struct Tm {
        tm_sec: c_int,
        tm_min: c_int,
        tm_hour: c_int,
        tm_mday: c_int,
        tm_mon: c_int,
        tm_year: c_int,
        tm_wday: c_int,
        tm_yday: c_int,
        tm_isdst: c_int,
        _rest: [c_long; 8],
    }

    extern "C" {
        fn localtime_r(t: *const i64, out: *mut Tm) -> *mut Tm;
    }

    pub fn local(secs: i64) -> Option<DateTime> {
        let mut tm = Tm { tm_sec: 0, tm_min: 0, tm_hour: 0, tm_mday: 0, tm_mon: 0, tm_year: 0, tm_wday: 0, tm_yday: 0, tm_isdst: 0, _rest: [0; 8] };
        // На всех 64-битных Unix `time_t` — 64-битное целое.
        let r = unsafe { localtime_r(&secs, &mut tm) };
        if r.is_null() {
            return None;
        }
        Some(DateTime {
            year: tm.tm_year as i64 + 1900,
            month: tm.tm_mon as u32 + 1,
            day: tm.tm_mday as u32,
            hour: tm.tm_hour as u32,
            minute: tm.tm_min as u32,
            second: tm.tm_sec as u32,
        })
    }
}

#[cfg(windows)]
mod imp {
    #![allow(non_snake_case)]
    use super::{utc_from_secs, DateTime};
    use std::os::raw::c_void;

    #[repr(C)]
    #[derive(Default)]
    struct SYSTEMTIME {
        wYear: u16,
        wMonth: u16,
        wDayOfWeek: u16,
        wDay: u16,
        wHour: u16,
        wMinute: u16,
        wSecond: u16,
        wMilliseconds: u16,
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn SystemTimeToTzSpecificLocalTime(tz: *const c_void, utc: *const SYSTEMTIME, local: *mut SYSTEMTIME) -> i32;
    }

    pub fn local(secs: i64) -> Option<DateTime> {
        let u = utc_from_secs(secs);
        if !(1601..=30827).contains(&u.year) {
            return None;
        }
        let utc = SYSTEMTIME {
            wYear: u.year as u16,
            wMonth: u.month as u16,
            wDayOfWeek: 0,
            wDay: u.day as u16,
            wHour: u.hour as u16,
            wMinute: u.minute as u16,
            wSecond: u.second as u16,
            wMilliseconds: 0,
        };
        let mut l = SYSTEMTIME::default();
        if unsafe { SystemTimeToTzSpecificLocalTime(std::ptr::null(), &utc, &mut l) } == 0 {
            return None;
        }
        Some(DateTime {
            year: l.wYear as i64,
            month: l.wMonth as u32,
            day: l.wDay as u32,
            hour: l.wHour as u32,
            minute: l.wMinute as u32,
            second: l.wSecond as u32,
        })
    }
}

#[cfg(not(any(unix, windows)))]
mod imp {
    pub fn local(_secs: i64) -> Option<super::DateTime> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utc() {
        assert_eq!(utc_from_secs(1709296440), DateTime { year: 2024, month: 3, day: 1, hour: 12, minute: 34, second: 0 });
        assert_eq!(utc_from_secs(0).year, 1970);
    }

    #[test]
    fn local_works() {
        // Проверяем только, что функция отвечает правдоподобно.
        let n = now();
        assert!(n.year >= 2024 && (1..=12).contains(&n.month) && n.hour < 24);
    }
}
