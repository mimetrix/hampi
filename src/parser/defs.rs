//! Top level handling of definitions

use crate::error::Error;
use crate::structs::{
    defs::{
        Asn1AssignmentKind, Asn1Definition, Asn1ObjectClassAssignment, Asn1ObjectSetAssignment,
        Asn1TypeAssignment, Asn1ValueAssignment, DefinitionParam, DummyReferenceKind, GovernerKind,
        ParamDummyReference, ParamGoverner,
    },
    ioc::Asn1ObjectSet,
};
use crate::tokenizer::Token;

use super::ioc::parse_class;
use super::types::parse_type;
use super::utils::{
    expect_keyword, expect_one_of_tokens, expect_token, expect_tokens, parse_set_ish_value,
};
use super::values::parse_value;

pub(super) fn parse_definition<'parser>(
    tokens: &'parser [Token],
) -> Result<(Asn1Definition, usize), Error> {
    let consumed = 0;

    if expect_one_of_tokens(
        &tokens[consumed..],
        &[Token::is_value_reference, Token::is_type_reference],
    )? {
        let next = &tokens[consumed];
        if next.is_value_reference() {
            parse_valueish_definition(&tokens[consumed..])
        } else {
            parse_typeish_definition(&tokens[consumed..])
        }
    } else {
        Err(parse_error!("Not Implemented!"))
    }
}

// Parse `ValueAssignment`, `ObjectAssignment` and `ParameterizedValueAssignment` etc.
//
// All the above assignments start with a lowe-case letter and will have to be parsed into their
// respective 'values'. Returns the corresponding variant of the `Asn1Definition` and  the number
// of tokens consumed or error.
fn parse_valueish_definition<'parser>(
    tokens: &'parser [Token],
) -> Result<(Asn1Definition, usize), Error> {
    let mut consumed = 0;

    if !expect_token(&tokens[consumed..], Token::is_value_reference)? {
        return Err(unexpected_token!("Value Reference", tokens[consumed]));
    }
    let id = tokens[consumed].text.clone();
    consumed += 1;

    let (typeref, typeref_consumed) = parse_type(&tokens[consumed..])?;
    consumed += typeref_consumed;

    if !expect_token(&tokens[consumed..], Token::is_assignment)? {
        return Err(unexpected_token!("::=", tokens[consumed]));
    }
    consumed += 1;

    let (value, value_consumed) = parse_value(&tokens[consumed..])?;
    consumed += value_consumed;

    Ok((
        Asn1Definition {
            kind: Asn1AssignmentKind::Value(Asn1ValueAssignment { id, typeref, value }),
            params: None,
        },
        consumed,
    ))
}

fn parse_typeish_definition<'parser>(
    tokens: &'parser [Token],
) -> Result<(Asn1Definition, usize), Error> {
    // Try to parse a type_definition
    match parse_type_definition(tokens) {
        Ok(x) => {
            return Ok(x);
        }
        Err(_) => {}
    }

    match parse_class_definition(tokens) {
        Ok(x) => {
            return Ok(x);
        }
        Err(_) => {}
    }

    match parse_object_set_definition(tokens) {
        Ok(x) => {
            return Ok(x);
        }
        Err(_) => {}
    }

    Err(parse_error!(
        "Failed to parse a definition at Token: {:#?}",
        tokens[0]
    ))
}

fn parse_type_definition<'parser>(
    tokens: &'parser [Token],
) -> Result<(Asn1Definition, usize), Error> {
    let mut consumed = 0;

    if !expect_token(&tokens[consumed..], Token::is_type_reference)? {
        return Err(unexpected_token!("Type Reference", tokens[consumed]));
    }
    let id = tokens[consumed].text.clone();
    consumed += 1;

    let (params, params_consumed) = match parse_params(&tokens[consumed..]) {
        Ok(result) => (Some(result.0), result.1),
        Err(_) => (None, 0),
    };
    consumed += params_consumed;

    if !expect_token(&tokens[consumed..], Token::is_assignment)? {
        return Err(unexpected_token!("::=", tokens[consumed]));
    }
    consumed += 1;

    let (typeref, typeref_consumed) = parse_type(&tokens[consumed..])?;
    consumed += typeref_consumed;

    Ok((
        Asn1Definition {
            kind: Asn1AssignmentKind::Type(Asn1TypeAssignment { id, typeref }),
            params,
        },
        consumed,
    ))
}

fn parse_class_definition<'parser>(
    tokens: &'parser [Token],
) -> Result<(Asn1Definition, usize), Error> {
    let mut consumed = 0;
    if !expect_token(&tokens[consumed..], Token::is_object_class_reference)? {
        return Err(unexpected_token!("CLASS Reference", tokens[consumed]));
    }
    let id = tokens[consumed].text.clone();
    consumed += 1;

    if !expect_token(&tokens[consumed..], Token::is_assignment)? {
        return Err(unexpected_token!("::=", tokens[consumed]));
    }
    consumed += 1;

    if !expect_keyword(&tokens[consumed..], "CLASS")? {
        return Err(unexpected_token!("'CLASS'", tokens[consumed]));
    }

    let (classref, classref_consumed) = parse_class(&tokens[consumed..])?;
    consumed += classref_consumed;

    Ok((
        Asn1Definition {
            kind: Asn1AssignmentKind::Class(Asn1ObjectClassAssignment { id, classref }),
            params: None,
        },
        consumed,
    ))
}

fn parse_object_set_definition<'parser>(
    tokens: &'parser [Token],
) -> Result<(Asn1Definition, usize), Error> {
    let mut consumed = 0;

    if !expect_token(&tokens[consumed..], Token::is_type_reference)? {
        return Err(unexpected_token!("'Type Reference'", tokens[consumed]));
    }
    let id = tokens[consumed].text.clone();
    consumed += 1;

    if !expect_token(&tokens[consumed..], Token::is_object_class_reference)? {
        return Err(unexpected_token!("'CLASS Reference'", tokens[consumed]));
    }
    let class = tokens[consumed].text.clone();
    consumed += 1;

    if !expect_token(&tokens[consumed..], Token::is_assignment)? {
        return Err(unexpected_token!("'::='", tokens[consumed]));
    }
    consumed += 1;

    if !expect_token(&tokens[consumed..], Token::is_curly_begin)? {
        return Err(unexpected_token!("'{'", tokens[consumed]));
    }
    consumed += 1;

    let mut objects = vec![];
    loop {
        if expect_token(&tokens[consumed..], Token::is_extension)? {
            consumed += 1;
            if expect_token(&tokens[consumed..], Token::is_extension)? {
                consumed += 1;
            }
        }

        match parse_set_ish_value(&tokens[consumed..]) {
            Ok(result) => {
                let (value, value_consumed) = result;
                objects.push(value);
                consumed += value_consumed;
            }
            Err(_) => {
                // It may be a reference to an object set, allowed
                if expect_one_of_tokens(
                    &tokens[consumed..],
                    &[Token::is_object_set_reference, Token::is_object_reference],
                )? {
                    objects.push(tokens[consumed].text.clone());
                    consumed += 1;
                }
            } // Empty Values permitted
        }

        if expect_token(&tokens[consumed..], Token::is_comma)? {
            consumed += 1;
        }

        if expect_token(&tokens[consumed..], Token::is_set_union)? {
            consumed += 1;
        }

        if expect_token(&tokens[consumed..], Token::is_curly_end)? {
            consumed += 1;
            break;
        }
    }

    Ok((
        Asn1Definition {
            kind: Asn1AssignmentKind::ObjectSet(Asn1ObjectSetAssignment {
                id,
                set: Asn1ObjectSet { class, objects },
            }),
            params: None,
        },
        consumed,
    ))
}

fn parse_params<'parser>(tokens: &'parser [Token]) -> Result<(Vec<DefinitionParam>, usize), Error> {
    let mut consumed = 0;

    if !expect_token(&tokens[consumed..], Token::is_curly_begin)? {
        return Err(unexpected_token!("'{'", tokens[consumed]));
    }
    consumed += 1;

    let mut params = vec![];
    loop {
        // Try to parse the Governer: DummyReference if fails, whatever remains is a Type or Class
        let (governer, dummyref) = if expect_tokens(
            &tokens[consumed..],
            &[
                &[
                    Token::is_type_reference,
                    Token::is_asn_builtin_type,
                    Token::is_object_class_reference,
                ],
                &[Token::is_colon],
                &[Token::is_value_reference, Token::is_type_reference],
            ],
        )? {
            let (g, r) = (&tokens[consumed], &tokens[consumed + 2]);
            let governer = if g.is_object_class_reference() {
                ParamGoverner {
                    name: g.text.clone(),
                    kind: GovernerKind::Class,
                }
            } else {
                ParamGoverner {
                    name: g.text.clone(),
                    kind: GovernerKind::Type,
                }
            };
            let dummyref = if r.is_value_reference() {
                if governer.kind == GovernerKind::Class {
                    ParamDummyReference {
                        name: r.text.clone(),
                        kind: DummyReferenceKind::Object,
                    }
                } else {
                    ParamDummyReference {
                        name: r.text.clone(),
                        kind: DummyReferenceKind::Value,
                    }
                }
            } else {
                if governer.kind == GovernerKind::Class {
                    ParamDummyReference {
                        name: r.text.clone(),
                        kind: DummyReferenceKind::ObjectSet,
                    }
                } else {
                    ParamDummyReference {
                        name: r.text.clone(),
                        kind: DummyReferenceKind::ValueSet,
                    }
                }
            };
            consumed += 3;
            (Some(governer), dummyref)
        } else if expect_one_of_tokens(
            &tokens[consumed..],
            &[Token::is_type_reference, Token::is_object_class_reference],
        )? {
            let r = &tokens[consumed];
            consumed += 1;
            if r.is_object_class_reference() {
                (
                    None,
                    ParamDummyReference {
                        name: r.text.clone(),
                        kind: DummyReferenceKind::Class,
                    },
                )
            } else {
                (
                    None,
                    ParamDummyReference {
                        name: r.text.clone(),
                        kind: DummyReferenceKind::Type,
                    },
                )
            }
        } else {
            return Err(parse_error!(
                "Error in Parsing params at Token: {:#?}",
                tokens[consumed]
            ));
        };
        let param = DefinitionParam { governer, dummyref };
        params.push(param);

        if expect_token(&tokens[consumed..], Token::is_comma)? {
            consumed += 1;
        }

        if expect_token(&tokens[consumed..], Token::is_curly_end)? {
            consumed += 1;
            break;
        }
    }

    Ok((params, consumed))
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::tokenizer::tokenize;

    #[test]
    fn parse_definition_tests() {
        struct ParseDefinitionTestCase<'tc> {
            input: &'tc str,
            success: bool,
        }

        let test_cases = vec![ParseDefinitionTestCase {
            input: r#"
PDCP-Capability-r4-ext ::=                      SEQUENCE {
        supportForRfc3095                               CHOICE {
                notSupported                                            NULL,
                supported                                                       SEQUENCE {
                        maxROHC-ContextSessions                         MaxROHC-ContextSessions-r4      DEFAULT s16,
                        reverseCompressionDepth                         INTEGER (0..65535)                      DEFAULT 0
                }
        }
}
                    "#,
            success: true,
        }];

        for tc in test_cases {
            let reader = std::io::BufReader::new(std::io::Cursor::new(tc.input));
            let tokens = tokenize(reader);
            assert!(tokens.is_ok());
            let tokens = tokens.unwrap();

            let def = parse_definition(&tokens);
            assert_eq!(
                def.is_ok(),
                tc.success,
                "{}:{}",
                tc.input,
                if tc.success {
                    format!("{:#?}", def.err())
                } else {
                    format!("{:#?}", def.ok())
                }
            );
        }

        assert!(true);
    }
}
