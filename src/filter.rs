use std::num::ParseFloatError;

use nom::{
  bytes::complete::{tag, take_till, take_until},
  character::{complete::char, is_alphabetic},
  combinator::map_res,
  error::Error,
  number::complete::float,
  Err, IResult,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseFilterError<'a> {
  #[error("{0}")]
  Nom(Err<Error<&'a str>>),
  #[error("{0}")]
  ParseFloatError(ParseFloatError),
  #[error("[`{0}`] is not valid unit")]
  UnitParseError(&'a str),
}

impl<'a> From<Err<Error<&'a str>>> for ParseFilterError<'a> {
  fn from(value: Err<Error<&'a str>>) -> Self {
    Self::Nom(value)
  }
}

impl<'a> From<ParseFloatError> for ParseFilterError<'a> {
  fn from(value: ParseFloatError) -> Self {
    Self::ParseFloatError(value)
  }
}

#[derive(Debug, PartialEq)]
pub enum CssFilter {
  Blur(f32),
  Brightness(f32),
  Contrast(f32),
}

#[inline(always)]
fn pixel(input: &str) -> Result<f32, ParseFilterError> {
  let (input, size) = take_till(|c| is_alphabetic(c as u8))(input)?;
  let (_, unit) = take_till(|c| c == ')')(input)?;
  let size = size.trim().parse::<f32>()?;
  let mut size_px = size;
  match unit.trim() {
    "em" | "rem" | "pc" => {
      size_px = size * 16.0;
    }
    "pt" => {
      size_px = size * 4.0 / 3.0;
    }
    "px" => {
      size_px = size;
    }
    "in" => {
      size_px = size * 96.0;
    }
    "cm" => {
      size_px = size * 96.0 / 2.54;
    }
    "mm" => {
      size_px = size * 96.0 / 25.4;
    }
    "q" => {
      size_px = size * 96.0 / 25.4 / 4.0;
    }
    "%" => {
      size_px = size * 16.0 / 100.0;
    }
    "" => {
      if size_px != 0f32 {
        return Err(ParseFilterError::UnitParseError("[No unit assigned]"));
      }
    }
    _ => {
      return Err(ParseFilterError::UnitParseError(unit));
    }
  };

  Ok(size_px)
}

#[inline(always)]
fn pixel_in_tuple(input: &str) -> IResult<&str, f32> {
  map_res(take_until(")"), pixel)(input)
}

#[inline(always)]
fn number_percentage(input: &str) -> IResult<&str, f32> {
  let (input, num) = float(input.trim())?;
  if let Ok((input, _)) = tag::<&str, &str, Error<&str>>("%")(input.trim()) {
    Ok((input, num / 100.0f32))
  } else {
    Ok((input, num))
  }
}

#[inline(always)]
pub fn css_filter(input: &str) -> IResult<&str, Vec<CssFilter>> {
  let mut filters = Vec::with_capacity(10);
  let mut input = input.trim();

  if let Ok((blurred_input, _)) = tag::<&str, &str, Error<&str>>("blur(")(input) {
    let (blurred_input, pixel) = pixel_in_tuple(blurred_input)?;
    let (finished_input, _) = char(')')(blurred_input)?;
    input = finished_input.trim();
    filters.push(CssFilter::Blur(pixel));
  }
  if let Ok((brightness_input, _)) = tag::<&str, &str, Error<&str>>("brightness(")(input) {
    let (brightness_input, brightness) = number_percentage(brightness_input)?;
    let (brightness_input, _) = char(')')(brightness_input.trim())?;
    input = brightness_input.trim();
    filters.push(CssFilter::Brightness(brightness));
  }
  Ok((input, filters))
}

#[test]
fn parse_empty() {
  assert_eq!(css_filter(""), Ok(("", vec![])));
}

#[test]
fn parse_blur() {
  assert_eq!(
    css_filter("blur(20px)"),
    Ok(("", vec![CssFilter::Blur(20.0)]))
  );
  assert_eq!(css_filter("blur(0)"), Ok(("", vec![CssFilter::Blur(0.0)])));
  assert_eq!(
    css_filter("blur(1.5rem)"),
    Ok(("", vec![CssFilter::Blur(24.0)]))
  );
  assert_eq!(
    css_filter("blur(20 px)"),
    Ok(("", vec![CssFilter::Blur(20.0)]))
  );
  assert_eq!(
    css_filter("blur( 20 px )"),
    Ok(("", vec![CssFilter::Blur(20.0)]))
  );
}

#[test]
fn parse_brightness() {
  assert_eq!(
    css_filter("brightness(2)"),
    Ok(("", vec![CssFilter::Brightness(2.0f32)]))
  );
  assert_eq!(
    css_filter("brightness(2%)"),
    Ok(("", vec![CssFilter::Brightness(0.02f32)]))
  );
  assert_eq!(
    css_filter("brightness( 2%)"),
    Ok(("", vec![CssFilter::Brightness(0.02f32)]))
  );
  assert_eq!(
    css_filter("brightness( 2% )"),
    Ok(("", vec![CssFilter::Brightness(0.02f32)]))
  );
  assert_eq!(
    css_filter("brightness( 2 % )"),
    Ok(("", vec![CssFilter::Brightness(0.02f32)]))
  );
  assert_eq!(
    css_filter(" brightness( 2 % )  "),
    Ok(("", vec![CssFilter::Brightness(0.02f32)]))
  );
}

#[test]
fn composite_parse() {
  assert_eq!(
    css_filter("blur(1.5rem) brightness(2)"),
    Ok((
      "",
      vec![CssFilter::Blur(24.0), CssFilter::Brightness(2.0f32)]
    ))
  );

  // assert_eq!(
  //   css_filter("brightness(2) blur(1.5rem)"),
  //   Ok((
  //     "",
  //     vec![CssFilter::Brightness(2.0f32), CssFilter::Blur(24.0)]
  //   ))
  // );
}

#[test]
fn parse_number_or_percentage() {
  assert_eq!(number_percentage("2"), Ok(("", 2f32)));
  assert_eq!(number_percentage("1.11"), Ok(("", 1.11f32)));
  assert_eq!(number_percentage("20%"), Ok(("", 0.2f32)));
  assert_eq!(number_percentage("-20%"), Ok(("", -0.2f32)));
  assert_eq!(number_percentage("-0.1"), Ok(("", -0.1f32)));
}
