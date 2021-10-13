use std::{thread, time::Duration};

use neon::prelude::*;

fn hello(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(cx.string("hello node"))
}

fn undef(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    Ok(cx.boolean(true))
}

// #[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("hello", hello)?;
    cx.export_function("undef", undef)?;
    // thread::sleep(Duration::from_secs(10));

    
    let boxed_fn = 
    cx.boxed(move |cx_: FunctionContext| -> JsResult<JsUndefined> { Ok(cx_.undefined()) });



    let jsfn = JsFunction::new(
        &mut cx,
        ,
    );
    // cx.export_value(
    //     "world",
    //     ,
    // )?;
    Ok(())
}
