use std::mem;

use napi::*;

use crate::sk::sk_svg_text_to_path;

#[js_function(1)]
pub fn convert_svg_text_to_path(ctx: CallContext) -> Result<JsBuffer> {
  let input = ctx.get::<JsBuffer>(0)?.into_value()?;
  sk_svg_text_to_path(&input)
    .ok_or_else(|| {
      Error::new(
        Status::InvalidArg,
        "Convert svg text to path failed".to_owned(),
      )
    })
    .and_then(|v| unsafe {
      ctx
        .env
        .create_buffer_with_borrowed_data(v.0.ptr, v.0.size, v, |d, _| mem::drop(d))
    })
    .map(|b| b.into_raw())
}
