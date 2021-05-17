use std::num::ParseFloatError;

use cssparser::{Color, Parser, ParserInput, RGBA};
use nom::{
  branch::alt,
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
  DropShadow(f32, f32, f32, RGBA),
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
fn brightness_parser(input: &str) -> IResult<&str, CssFilter> {
  let (brightness_input, _) = tag("brightness(")(input)?;
  let (brightness_input, brightness) = number_percentage(brightness_input)?;
  let (brightness_input, _) = char(')')(brightness_input.trim())?;
  Ok((brightness_input.trim(), CssFilter::Brightness(brightness)))
}

#[inline(always)]
fn blur_parser(input: &str) -> IResult<&str, CssFilter> {
  let (blurred_input, _) = tag("blur(")(input)?;

  let (blurred_input, pixel) = pixel_in_tuple(blurred_input)?;
  let (finished_input, _) = char(')')(blurred_input)?;
  Ok((finished_input.trim(), CssFilter::Blur(pixel)))
}

#[inline(always)]
fn contrast_parser(input: &str) -> IResult<&str, CssFilter> {
  let (contrast_input, _) = tag("contrast(")(input)?;
  let (contrast_input, contrast) = number_percentage(contrast_input)?;
  let (contrast_input, _) = char(')')(contrast_input.trim())?;
  Ok((contrast_input.trim(), CssFilter::Contrast(contrast)))
}

#[inline(always)]
fn parse_drop_shadow(input: &str) -> IResult<&str, CssFilter> {
  let (drop_shadow_input, _) = tag("drop-shadow(")(input)?;
  let drop_shadow_input = drop_shadow_input.trim();
  let (offset_x_output, offset_x) = map_res(take_until(" "), pixel)(drop_shadow_input)?;
  let offset_x_output = offset_x_output.trim();
  let (offset_y_output, offset_y) =
    map_res(take_till(|ch| ch == ' ' || ch == ')'), pixel)(offset_x_output)?;
  let offset_y_output = offset_y_output.trim();
  let (blur_radius_output, blur_radius) =
    map_res(take_till(|ch| ch == ' ' || ch == ')'), pixel)(offset_y_output)
      .unwrap_or_else(|_: Err<Error<&str>>| (offset_y_output, 0.0f32));
  let blur_radius_output = blur_radius_output.trim();
  let is_rgb_fn = blur_radius_output.starts_with("rgb(") || blur_radius_output.starts_with("rgba(");
  let (shadow_color_output, shadow_color_str) =
    take_until(if is_rgb_fn { "))" } else { ")" })(blur_radius_output)?;
  let shadow_color_str = shadow_color_str.trim();
  static BLACK: RGBA = RGBA {
    red: 0,
    green: 0,
    blue: 0,
    alpha: 255,
  };
  let shadow_color = if !shadow_color_str.is_empty() {
    let mut parser_input = ParserInput::new(shadow_color_str);
    let mut parser = Parser::new(&mut parser_input);
    let color = Color::parse(&mut parser).unwrap_or_else(|_| Color::RGBA(BLACK));
    if let Color::RGBA(rgba) = color {
      rgba
    } else {
      BLACK
    }
  } else {
    BLACK
  };
  let (mut drop_shadow_output, _) = char(')')(shadow_color_output.trim())?;
  if is_rgb_fn {
    let (trimmed_drop_shadow_output, _) = char(')')(drop_shadow_output)?;
    drop_shadow_output = trimmed_drop_shadow_output;
  }
  Ok((
    drop_shadow_output.trim(),
    CssFilter::DropShadow(offset_x, offset_y, blur_radius, shadow_color),
  ))
}

#[inline(always)]
pub fn css_filter(input: &str) -> IResult<&str, Vec<CssFilter>> {
  let mut filters = Vec::with_capacity(10);
  let mut input = input.trim();
  while let Ok((output, filter)) = alt((
    blur_parser,
    brightness_parser,
    contrast_parser,
    parse_drop_shadow,
  ))(input)
  {
    input = output;
    filters.push(filter);
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
fn drop_shadow_parse() {
  assert_eq!(
    parse_drop_shadow("drop-shadow(2px 2px)"),
    Ok((
      "",
      CssFilter::DropShadow(2.0f32, 2.0f32, 0.0f32, RGBA::new(0, 0, 0, 255))
    ))
  );
  assert_eq!(
    parse_drop_shadow("drop-shadow(2px 2px 5px)"),
    Ok((
      "",
      CssFilter::DropShadow(2.0f32, 2.0f32, 5.0f32, RGBA::new(0, 0, 0, 255))
    ))
  );

  assert_eq!(
    parse_drop_shadow("drop-shadow(2px 2px 5px #2F14DF)"),
    Ok((
      "",
      CssFilter::DropShadow(2.0f32, 2.0f32, 5.0f32, RGBA::new(47, 20, 223, 255))
    ))
  );

  assert_eq!(
    parse_drop_shadow("drop-shadow(2px 2px 5px rgba(47, 20, 223, 255))"),
    Ok((
      "",
      CssFilter::DropShadow(2.0f32, 2.0f32, 5.0f32, RGBA::new(47, 20, 223, 255))
    ))
  );
}

#[test]
fn contrast_parse() {
  assert_eq!(
    css_filter("contrast(200%)"),
    Ok(("", vec![CssFilter::Contrast(2.0f32)]))
  );
  assert_eq!(
    css_filter("contrast( 200%)"),
    Ok(("", vec![CssFilter::Contrast(2.0f32)]))
  );
  assert_eq!(
    css_filter("contrast(200% )"),
    Ok(("", vec![CssFilter::Contrast(2.0f32)]))
  );
  assert_eq!(
    css_filter("contrast( 200% )"),
    Ok(("", vec![CssFilter::Contrast(2.0f32)]))
  );
  assert_eq!(
    css_filter("contrast( 200% )  "),
    Ok(("", vec![CssFilter::Contrast(2.0f32)]))
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

  assert_eq!(
    css_filter("brightness(2) blur(1.5rem)"),
    Ok((
      "",
      vec![CssFilter::Brightness(2.0f32), CssFilter::Blur(24.0)]
    ))
  );

  assert_eq!(
    css_filter("drop-shadow(2px 2px 5px rgba(47, 20, 223, 255)) brightness(2) blur(1.5rem)"),
    Ok((
      "",
      vec![
        CssFilter::DropShadow(2.0f32, 2.0f32, 5.0f32, RGBA::new(47, 20, 223, 255)),
        CssFilter::Brightness(2.0f32),
        CssFilter::Blur(24.0)
      ]
    ))
  );

  assert_eq!(
    css_filter("brightness(2) drop-shadow(2px 2px 5px rgba(47, 20, 223, 255)) blur(1.5rem)"),
    Ok((
      "",
      vec![
        CssFilter::Brightness(2.0f32),
        CssFilter::DropShadow(2.0f32, 2.0f32, 5.0f32, RGBA::new(47, 20, 223, 255)),
        CssFilter::Blur(24.0)
      ]
    ))
  );

  assert_eq!(
    css_filter("brightness(2) blur(1.5rem) drop-shadow(2px 2px 5px rgba(47, 20, 223, 255))"),
    Ok((
      "",
      vec![
        CssFilter::Brightness(2.0f32),
        CssFilter::Blur(24.0),
        CssFilter::DropShadow(2.0f32, 2.0f32, 5.0f32, RGBA::new(47, 20, 223, 255)),
      ]
    ))
  );
}

#[test]
fn parse_number_or_percentage() {
  assert_eq!(number_percentage("2"), Ok(("", 2f32)));
  assert_eq!(number_percentage("1.11"), Ok(("", 1.11f32)));
  assert_eq!(number_percentage("20%"), Ok(("", 0.2f32)));
  assert_eq!(number_percentage("-20%"), Ok(("", -0.2f32)));
  assert_eq!(number_percentage("-0.1"), Ok(("", -0.1f32)));
}
