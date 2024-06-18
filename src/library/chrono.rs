use std::{collections::HashMap, sync::Arc};

use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, Timelike, Utc};

use crate::{
    context::Context,
    error::Error,
    expr::{annotate_type, Expr},
    util::{
        args::{unpack_int_arg, unpack_map_arg},
        module_util::require_module,
    },
};

// #todo support rfc-399 and rfc-2822

// #link https://datatracker.ietf.org/doc/html/rfc3339
// #link https://ijmacd.github.io/rfc3339-iso8601/

// #todo what is a better name for DateTime.
// #todo does it make sense to support a Date? maybe we could only support a 'DateTime'
// #todo consider another namespace?
// #todo consider adding some chrono types to the prelude.
// #todo register the`Date` and `Duration` types.

// #insight `Duration` is similar to `Time`, i.e. time is a 'duration' from 0000-00-00, explore this.
// #todo Range instead of Duration?

pub fn tan_date_time_from_rust_date_time(rust_date_time: NaiveDateTime) -> Expr {
    // #todo month0, day0 is an interesting idea.
    let mut map = HashMap::new();
    // #todo add helpers to initialize Expr::Int
    map.insert("year".to_string(), Expr::Int(rust_date_time.year() as i64));
    map.insert(
        "month".to_string(),
        Expr::Int((rust_date_time.month0() + 1) as i64),
    );
    map.insert(
        "day".to_string(),
        Expr::Int((rust_date_time.day0() + 1) as i64),
    );

    map.insert("hour".to_string(), Expr::Int(rust_date_time.hour() as i64));
    map.insert(
        "minute".to_string(),
        Expr::Int(rust_date_time.minute() as i64),
    );
    map.insert(
        "second".to_string(),
        Expr::Int(rust_date_time.second() as i64),
    );

    // #todo support annotation with multiple types/traits, e.g. both Date + Map.

    let expr = Expr::map(map);

    annotate_type(expr, "Date-Time")
}

pub fn chrono_date_time_now(_args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let date_time = Utc::now().naive_utc();
    Ok(tan_date_time_from_rust_date_time(date_time))
}

// #todo add unit test
pub fn chrono_date_time(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    if args.is_empty() {
        chrono_date_time_now(args, _context)
    } else {
        todo!();
        // chrono_date_from_string(args, _context)
    }
}

pub fn chrono_date_time_to_string(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [this] = args else {
        return Err(Error::invalid_arguments("requires `this` argument", None));
    };

    // #todo check dyn_type.

    let Some(map) = this.as_map() else {
        return Err(Error::invalid_arguments(
            "`this` argument should be a Date-Time",
            this.range(),
        ));
    };

    // #todo error checking!

    let Some(year) = map["year"].as_int() else {
        return Err(Error::invalid_arguments("invalid Date-Time", this.range()));
    };

    let Some(month) = map["month"].as_int() else {
        return Err(Error::invalid_arguments("invalid Date-Time", this.range()));
    };

    let Some(day) = map["day"].as_int() else {
        return Err(Error::invalid_arguments("invalid Dat-Time", this.range()));
    };

    let Some(hour) = map["hour"].as_int() else {
        return Err(Error::invalid_arguments("invalid Date-Time", this.range()));
    };

    let Some(minute) = map["minute"].as_int() else {
        return Err(Error::invalid_arguments("invalid Date-Time", this.range()));
    };

    let Some(second) = map["second"].as_int() else {
        return Err(Error::invalid_arguments("invalid Date-Time", this.range()));
    };

    // #todo have separate function for to-rfc-399
    let str = format!(
        "{}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hour, minute, second
    );

    Ok(Expr::string(str))
}

// #todo differentiate to_string from to_rfc399
pub fn chrono_date_time_to_rfc399(args: &[Expr], context: &mut Context) -> Result<Expr, Error> {
    chrono_date_time_to_string(args, context)
}

pub fn tan_date_from_components(year: i64, month: i64, day: i64) -> Expr {
    // #todo month0, day0 is an interesting idea.
    let mut map = HashMap::new();
    // #todo add helpers to initialize Expr::Int
    map.insert("year".to_string(), Expr::Int(year));
    map.insert("month".to_string(), Expr::Int(month));
    map.insert("day".to_string(), Expr::Int(day));

    // #todo support annotation with multiple types/traits, e.g. both Date + Map.

    let expr = Expr::map(map);

    annotate_type(expr, "Date")
}

pub fn tan_date_from_rust_date(rust_date: NaiveDate) -> Expr {
    tan_date_from_components(
        rust_date.year() as i64,
        (rust_date.month0() + 1) as i64,
        (rust_date.day0() + 1) as i64,
    )
    // // #todo month0, day0 is an interesting idea.
    // let mut map = HashMap::new();
    // // #todo add helpers to initialize Expr::Int
    // map.insert("year".to_string(), Expr::Int(rust_date.year() as i64));
    // map.insert(
    //     "month".to_string(),
    //     Expr::Int((rust_date.month0() + 1) as i64),
    // );
    // map.insert("day".to_string(), Expr::Int((rust_date.day0() + 1) as i64));

    // // #todo support annotation with multiple types/traits, e.g. both Date + Map.

    // let expr = Expr::map(map);

    // annotate_type(expr, "Date")
}

// #todo as un-optimized as it gets.
pub fn rust_date_from_tan_date(tan_date: &Expr) -> NaiveDate {
    let map = tan_date.as_map().unwrap();
    let s = format!("{}-{}-{}", map["year"], map["month"], map["day"]);
    let format_string = "%Y-%m-%d";
    // NaiveDate::from_ymd_opt(2024, 1, 18).unwrap();
    NaiveDate::parse_from_str(&s, format_string).unwrap()
}

// #insight i64s used to match Expr::Int()

// // #ai
// /// Returns true if the input is a leap year.
// /// Leap year logic: divisible by 4 but not divisible by 100 unless also divisible by 400
// fn is_leap_year(year: i64) -> bool {
//     (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
// }

// // #ai
// fn days_in_month(month: i64, year: i64) -> i64 {
//     match month {
//         1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
//         4 | 6 | 9 | 11 => 30,
//         2 => {
//             if is_leap_year(year) {
//                 29
//             } else {
//                 28
//             }
//         }
//         _ => panic!("Invalid month"),
//     }
// }

pub fn chrono_date_now(_args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let date = Utc::now().naive_utc().date();
    Ok(tan_date_from_rust_date(date))
}

// #todo support construction from [year month day]

// #todo support optional format string.
pub fn chrono_date_from_string(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [this] = args else {
        return Err(Error::invalid_arguments("requires `str` argument", None));
    };

    let Expr::String(s) = this.unpack() else {
        return Err(Error::invalid_arguments(
            "`str` argument should be a String",
            this.range(),
        ));
    };

    // #todo make customizable.
    let format_string = "%Y-%m-%d";

    let Ok(date) = NaiveDate::parse_from_str(s, format_string) else {
        return Err(Error::invalid_arguments("invalid date string", None));
    };

    Ok(tan_date_from_rust_date(date))
}

pub fn chrono_date(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    if args.is_empty() {
        chrono_date_now(args, _context)
    } else {
        chrono_date_from_string(args, _context)
    }
}

pub fn chrono_date_from_components(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let year = unpack_int_arg(args, 0, "year")?;
    let month = unpack_int_arg(args, 1, "month")?;
    let day = unpack_int_arg(args, 2, "day")?;
    Ok(tan_date_from_components(year, month, day))
}

pub fn chrono_date_from_map(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let map = unpack_map_arg(args, 0, "components")?;
    let expr = Expr::map(map.clone());
    Ok(annotate_type(expr, "Date"))
}

pub fn chrono_date_to_string(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [this] = args else {
        return Err(Error::invalid_arguments("requires `this` argument", None));
    };

    // #todo check dyn_type.

    let Some(map) = this.as_map() else {
        return Err(Error::invalid_arguments(
            "`this` argument should be a Date",
            this.range(),
        ));
    };

    // #todo error checking!

    let Some(year) = map["year"].as_int() else {
        return Err(Error::invalid_arguments("invalid Date", this.range()));
    };

    let Some(month) = map["month"].as_int() else {
        return Err(Error::invalid_arguments("invalid Date", this.range()));
    };

    let Some(day) = map["day"].as_int() else {
        return Err(Error::invalid_arguments("invalid Date", this.range()));
    };
    let str = format!("{}-{:02}-{:02}", year, month, day);

    Ok(Expr::string(str))
}

// #todo implement me!
pub fn chrono_date_to_rfc399(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [this] = args else {
        return Err(Error::invalid_arguments("requires `this` argument", None));
    };

    // #todo check dyn_type.

    let Some(map) = this.as_map() else {
        return Err(Error::invalid_arguments(
            "`this` argument should be a Date",
            this.range(),
        ));
    };

    // #todo error checking!

    let Some(year) = map["year"].as_int() else {
        return Err(Error::invalid_arguments("invalid Date", this.range()));
    };

    let Some(month) = map["month"].as_int() else {
        return Err(Error::invalid_arguments("invalid Date", this.range()));
    };

    let Some(day) = map["day"].as_int() else {
        return Err(Error::invalid_arguments("invalid Date", this.range()));
    };

    let str = format!("{}-{:02}-{:02}T00:00:00", year, month, day);

    Ok(Expr::string(str))
}

// https://docs.rs/chrono/latest/chrono/format/strftime/
pub fn chrono_date_format(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [spec, date] = args else {
        return Err(Error::invalid_arguments(
            "requires `spec` and `date` arguments",
            None,
        ));
    };

    let rust_date = rust_date_from_tan_date(date);

    let Some(fmt) = spec.as_stringable() else {
        return Err(Error::invalid_arguments(
            "`format-spec` argument should be a Stringable",
            spec.range(),
        ));
    };

    let output = rust_date.format(fmt);

    Ok(Expr::string(output.to_string()))
}

pub fn chrono_date_add_days(args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let [this, days] = args else {
        return Err(Error::invalid_arguments("requires `this` argument", None));
    };

    // #todo check dyn_type.

    let Some(days) = days.as_int() else {
        return Err(Error::invalid_arguments(
            "`days` argument should be an Int",
            this.range(),
        ));
    };

    let rust_date = rust_date_from_tan_date(this);

    let new_rust_date = rust_date + Duration::days(days);

    Ok(tan_date_from_rust_date(new_rust_date))
}

pub fn setup_lib_chrono(context: &mut Context) {
    let module = require_module("chrono", context);

    module.insert("Date-Time", Expr::ForeignFunc(Arc::new(chrono_date_time)));
    // #todo consider `to-rfc-399-string` or `format-rfc-399`
    module.insert(
        "to-rfc-399",
        Expr::ForeignFunc(Arc::new(chrono_date_time_to_rfc399)),
    );
    module.insert(
        "to-rfc-399$$Date-Time",
        Expr::ForeignFunc(Arc::new(chrono_date_time_to_rfc399)),
    );
    // #todo consider (String date-time)
    // #insight #hack this is added in prelude! NASTY hack
    // module.insert(
    //     "to-string$$Date-Time",
    //     Expr::ForeignFunc(Arc::new(chrono_date_time_to_string)),
    // );

    module.insert("Date", Expr::ForeignFunc(Arc::new(chrono_date)));
    module.insert(
        "Date$$Map",
        Expr::ForeignFunc(Arc::new(chrono_date_from_map)),
    );
    module.insert(
        "Date$$Int$$Int$$Int",
        Expr::ForeignFunc(Arc::new(chrono_date_from_components)),
    );

    // #todo implement with duration and `+`.
    module.insert(
        "add-days",
        Expr::ForeignFunc(Arc::new(chrono_date_add_days)),
    );
    // #insight spec comes first for more 'natural' currying.
    // #todo maybe just pass optional parameters to to-string?
    // #todo what would be a better name? stringf, strfmt? format is just too generic to reserve.
    module.insert(
        "format-string",
        Expr::ForeignFunc(Arc::new(chrono_date_format)),
    );
    // #todo add more functions
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;

    use crate::{api::eval_string, context::Context, expr::Expr};

    use super::chrono_date;

    #[test]
    fn chrono_date_usage() {
        let mut context = Context::new();
        let args = [Expr::string("2024-01-17")];
        let date = chrono_date(&args, &mut context).unwrap();
        let map = date.as_map().unwrap();
        assert_matches!(map.get("year").unwrap(), Expr::Int(year) if *year == 2024);
        assert_matches!(map.get("month").unwrap(), Expr::Int(month) if *month == 1);
        assert_matches!(map.get("day").unwrap(), Expr::Int(day) if *day == 17);
    }

    #[test]
    fn chrono_date_add_days_usage() {
        let mut context = Context::new();
        let input = r#"
            (use [Date add-days] chrono)
            (let d (Date "2024-01-18"))
            (let d (add-days d 2))
            (to-string d)
        "#;
        let expr = eval_string(input, &mut context).unwrap();
        assert_matches!(expr, Expr::String(s) if s == "2024-01-20");

        let input = r#"
            (use [Date add-days] chrono)
            (let d (Date "2024-01-18"))
            (let d (add-days d -20))
            (to-string d)
        "#;
        let expr = eval_string(input, &mut context).unwrap();
        assert_matches!(expr, Expr::String(s) if s == "2023-12-29");
    }

    #[test]
    fn date_format_usage() {
        let mut context = Context::new();
        let input = r#"
            (use [Date format-string] chrono)
            (let d (Date "2024-01-18"))
            (format-string "%B %d, %Y" d)
        "#;
        let expr = eval_string(input, &mut context).unwrap();
        assert_matches!(expr, Expr::String(s) if s == "January 18, 2024");
    }
}
