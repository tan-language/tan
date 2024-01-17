use std::{collections::HashMap, sync::Arc};

use chrono::{Datelike, NaiveDate, Utc};

use crate::{
    context::Context,
    error::Error,
    expr::{annotate_type, Expr},
    util::module_util::require_module,
};

// #todo what is a better name for DateTime.
// #todo does it make sense to support a Date? maybe we could only support a 'DateTime'
// #todo consider another namespace?
// #todo consider adding some chrono types to the prelude.

pub fn tan_date_from_rust_date(rust_date: NaiveDate) -> Expr {
    // #todo month0, day0 is an interesting idea.
    let mut dict = HashMap::new();
    // #todo add helpers to initialize Expr::Int
    dict.insert("year".to_string(), Expr::Int(rust_date.year() as i64));
    dict.insert(
        "month".to_string(),
        Expr::Int((rust_date.month0() + 1) as i64),
    );
    dict.insert("day".to_string(), Expr::Int((rust_date.day0() + 1) as i64));

    // #todo support annotation with multiple types/traits, e.g. both Date + Map.

    let expr = Expr::dict(dict);
    annotate_type(expr, "Date")
}

pub fn chrono_date_now(_args: &[Expr], _context: &mut Context) -> Result<Expr, Error> {
    let date = Utc::now().naive_utc().date();
    Ok(tan_date_from_rust_date(date))
}

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

pub fn setup_lib_chrono(context: &mut Context) {
    let module = require_module("chrono", context);

    module.insert("Date", Expr::ForeignFunc(Arc::new(chrono_date)));
    // #todo add more functions
}
