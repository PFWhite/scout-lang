use std::sync::Arc;

use fantoccini::{
    actions::{InputSource, KeyAction, KeyActions},
    elements::Element,
    key::Key,
};
use futures::{future::BoxFuture, FutureExt, TryFutureExt};

use crate::{object::Object, EvalError, EvalResult, ScrapeResultsPtr};

macro_rules! assert_param_len {
    ($arg:expr, $len:expr) => {
        if $arg.len() < $len {
            return Err($crate::EvalError::InvalidFnParams);
        }
    };
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BuiltinKind {
    Print,
    TextContent,
    Href,
    Trim,
    Click,
    Results,
    Len,
    Input,
    Contains,
    Type,
    KeyPress,
}

impl BuiltinKind {
    pub fn is_from(s: &str) -> Option<Self> {
        use BuiltinKind::*;
        match s {
            "print" => Some(Print),
            "textContent" => Some(TextContent),
            "trim" => Some(Trim),
            "href" => Some(Href),
            "click" => Some(Click),
            "results" => Some(Results),
            "len" => Some(Len),
            "input" => Some(Input),
            "contains" => Some(Contains),
            "type" => Some(Type),
            "key_action" => Some(KeyPress),
            _ => None,
        }
    }

    pub async fn apply(
        &self,
        crawler: &fantoccini::Client,
        results: ScrapeResultsPtr,
        args: Vec<Arc<Object>>,
    ) -> EvalResult {
        use BuiltinKind::*;
        match self {
            Type => {
                assert_param_len!(args, 1);
                Ok(Arc::new(Object::Str(args[0].type_str().to_string())))
            }
            Print => {
                for obj in args {
                    println!("{obj}");
                }
                Ok(Arc::new(Object::Null))
            }
            TextContent => {
                assert_param_len!(args, 1);
                apply_elem_fn(&args[0], |elem| {
                    async move { Object::Str(elem.text().await.unwrap_or("".into())) }.boxed()
                })
                .await
            }
            Href => {
                assert_param_len!(args, 1);
                apply_elem_fn(&args[0], |elem| {
                    async move {
                        Object::Str(
                            elem.prop("href")
                                .await
                                .unwrap_or(Option::None)
                                .unwrap_or("".into()),
                        )
                    }
                    .boxed()
                })
                .await
            }
            Click => {
                assert_param_len!(args, 1);
                if let Object::Node(elem) = &*args[0] {
                    let _ = elem.click().await;
                    Ok(Arc::new(Object::Null))
                } else {
                    Err(EvalError::InvalidFnParams)
                }
            }
            Trim => {
                assert_param_len!(args, 1);
                if let Object::Str(s) = &*args[0] {
                    Ok(Arc::new(Object::Str(s.trim().to_owned())))
                } else {
                    Err(EvalError::InvalidFnParams)
                }
            }
            Results => {
                let json = results.lock().await.to_json();
                println!("{}", json);
                Ok(Arc::new(Object::Null))
            }
            Len => {
                assert_param_len!(args, 1);
                let len = match &*args[0] {
                    Object::List(v) => Ok(v.len() as f64),
                    Object::Str(s) => Ok(s.len() as f64),
                    _ => Err(EvalError::InvalidFnParams),
                }?;

                Ok(Arc::new(Object::Number(len)))
            }
            KeyPress => {
                assert_param_len!(args, 1);
                if let Object::Str(code_str) = &*args[0] {
                    let code = code_str.chars().next().ok_or(EvalError::InvalidFnParams)?;
                    let actions = KeyActions::new("keypress".to_owned())
                        .then(KeyAction::Down { value: code });

                    crawler.perform_actions(actions).await?;
                    Ok(Arc::new(Object::Null))
                } else {
                    Err(EvalError::InvalidFnParams)
                }
            }
            Input => {
                assert_param_len!(args, 2);
                match (&*args[0], &*args[1]) {
                    (Object::Node(elem), Object::Str(s)) => {
                        elem.send_keys(s)
                            .map_err(|_| EvalError::BrowserError)
                            .await?;

                        if args.len() > 2 && args[2].is_truthy() {
                            let actions =
                                KeyActions::new("enter".to_owned()).then(KeyAction::Down {
                                    value: Key::Return.into(),
                                });
                            crawler.perform_actions(actions).await?;
                        }
                        Ok(Arc::new(Object::Null))
                    }
                    _ => Err(EvalError::InvalidFnParams),
                }
            }
            Contains => {
                assert_param_len!(args, 2);
                match &*args[0] {
                    Object::Str(s) => match &*args[1] {
                        Object::Str(sub) => {
                            let contains = s.contains(sub);
                            Ok(Arc::new(Object::Boolean(contains)))
                        }
                        _ => Err(EvalError::InvalidFnParams),
                    },
                    Object::List(v) => {
                        let contains = v.contains(&args[1]);
                        Ok(Arc::new(Object::Boolean(contains)))
                    }
                    _ => Err(EvalError::InvalidFnParams),
                }
            }
        }
    }
}

async fn apply_elem_fn(
    arg: &Object,
    f: impl Fn(&'_ Element) -> BoxFuture<'_, Object>,
) -> EvalResult {
    match arg {
        Object::Node(elem) => Ok(Arc::new(f(elem).await)),
        Object::List(list) => {
            let mut res = Vec::new();
            for obj in list.iter() {
                if let Object::Node(elem) = &*obj.clone() {
                    res.push(Arc::new(f(elem).await));
                } else {
                    return Err(EvalError::InvalidUsage(
                        "cannot run builtin node fn against non-node".into(),
                    ));
                }
            }
            Ok(Arc::new(Object::List(res)))
        }
        _ => Err(EvalError::InvalidFnParams),
    }
}
