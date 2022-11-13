use web_sys::HtmlCanvasElement;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

pub fn get_canvas(name: &str) -> Option<web_sys::HtmlCanvasElement> {
    let document = web_sys::window()?.document()?;
    let canvas = document.get_element_by_id(name)?;

    canvas.dyn_into::<web_sys::HtmlCanvasElement>().ok()
}

pub fn get_ctx<T : JsCast>(canvas_name: &str, ctx_name: &str) -> Result<T, JsValue> {
    let canvas = get_canvas(canvas_name)
        .ok_or_else(|| JsValue::from_str("Failed to get canvas"))?;

    get_ctx_from_canvas(&canvas, ctx_name)
}

pub fn get_ctx_from_canvas<T: JsCast>(canvas: &HtmlCanvasElement, ctx_name: &str) -> Result<T, JsValue> {
    let ctx = canvas
        .get_context(ctx_name)?
        .ok_or_else(|| JsValue::from_str("Failed getting ctx"))?;

    ctx.dyn_into::<T>()
        .map_err(JsValue::from)
}