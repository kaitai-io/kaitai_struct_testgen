use crate::ast::{BinaryOp, Expr, UnaryOp};

pub fn translate(expr: &Expr) -> String {
    match expr {
        Expr::Int(x) => x.to_string(),
        Expr::Float(x) => {
            let value = x.value();
            let formatted = if should_format_float_with_exponent(value) {
                format!("{:e}", value)
            } else {
                value.to_string()
            };
            if formatted.chars().all(|ch| ch.is_ascii_digit()) {
                // The float has been formatted as a valid integer, which means that KSC would
                // interpret it as an integer if we leave it as is. But we don't want that - this
                // AST node represents a float and it must remain this way.
                formatted + ".0"
            } else {
                formatted
            }
        }
        Expr::Str(x) => {
            // See https://doc.kaitai.io/user_guide.html#_basic_data_types:
            // > Everything between single quotes is interpreted literally, i.e. there is no way one
            // > can include a single quote inside a single quoted string.
            assert!(
                !x.contains('\''),
                "strings containing a single quote (') not supported yet (got {})",
                x
            );
            format!("'{}'", x)
        }
        Expr::Bool(x) => x.to_string(),
        Expr::EnumMember { enum_path, label } => {
            let mut parts: Vec<&str> = enum_path.iter().map(|s| s.as_str()).collect();
            parts.push(label);
            parts.join("::")
        }
        Expr::List(items) => format!(
            "[{}]",
            items.iter().map(translate).collect::<Vec<_>>().join(", ")
        ),

        Expr::Name(name) => name.clone(),
        Expr::Attribute { value, attr_name } => format!("{}.{}", translate(value), attr_name),
        Expr::MethodCall {
            value,
            method_name,
            args,
        } => format!(
            "{}.{}({})",
            translate(value),
            method_name,
            args.iter().map(translate).collect::<Vec<_>>().join(", ")
        ),

        Expr::UnaryOp { op, v } => format!("({}{})", translate_unary_op(op), translate(v)),
        Expr::BinaryOp { l, op, r } => format!(
            "({} {} {})",
            translate(l),
            translate_binary_op(op),
            translate(r)
        ),
        Expr::CondOp {
            cond,
            if_true,
            if_false,
        } => format!(
            "({} ? {} : {})",
            translate(cond),
            translate(if_true),
            translate(if_false)
        ),
        Expr::Subscript { value, idx } => format!("{}[{}]", translate(value), translate(idx)),
    }
}

fn translate_unary_op(op: &UnaryOp) -> &'static str {
    match op {
        UnaryOp::Neg => "-",
        UnaryOp::Not => "not ",
        UnaryOp::Inv => "~",
    }
}

fn translate_binary_op(op: &BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Sub => "-",
        BinaryOp::Mul => "*",
        BinaryOp::Div => "/",
        BinaryOp::Rem => "%",
        BinaryOp::Eq => "==",
        BinaryOp::Ne => "!=",
        BinaryOp::Lt => "<",
        BinaryOp::Le => "<=",
        BinaryOp::Gt => ">",
        BinaryOp::Ge => ">=",
        BinaryOp::And => "and",
        BinaryOp::Or => "or",
        BinaryOp::BitOr => "|",
        BinaryOp::BitXor => "^",
        BinaryOp::BitAnd => "&",
        BinaryOp::Shl => "<<",
        BinaryOp::Shr => ">>",
    }
}

fn should_format_float_with_exponent(value: f64) -> bool {
    if value == 0.0 {
        false
    } else {
        !(1e-4..1e16).contains(&value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::utils::PositiveFiniteF64;
    use num_bigint::BigUint;

    #[test]
    fn int() {
        let expr = Expr::Int(BigUint::from(u64::MAX));
        assert_eq!(translate(&expr), "18446744073709551615");
    }

    #[test]
    fn float() {
        let expr = Expr::Float(PositiveFiniteF64::try_from(std::f64::consts::PI).unwrap());
        assert_eq!(translate(&expr), "3.141592653589793");
    }

    #[test]
    fn float_zero() {
        let expr = Expr::Float(PositiveFiniteF64::try_from(0.0).unwrap());
        assert_eq!(translate(&expr), "0.0");
    }

    #[test]
    fn float_exact_int() {
        let expr = Expr::Float(PositiveFiniteF64::try_from(13.0).unwrap());
        assert_eq!(translate(&expr), "13.0");
    }

    #[test]
    fn float_pos_exp_max_only_digits() {
        let value: f64 = 9_99999_99999_99998.0;
        assert_eq!(value.to_bits(), 0x4341_C379_37E0_7FFF_u64);
        let expr = Expr::Float(PositiveFiniteF64::try_from(value).unwrap());
        assert_eq!(translate(&expr), "9999999999999998.0");
    }

    #[test]
    fn float_pos_exp_min_with_exponent() {
        let value: f64 = 10_00000_00000_00000.0;
        assert_eq!(value.to_bits(), 0x4341_C379_37E0_8000_u64);
        let expr = Expr::Float(PositiveFiniteF64::try_from(value).unwrap());
        assert_eq!(translate(&expr), "1e16");
    }

    #[test]
    fn float_neg_exp_min_only_digits() {
        let value: f64 = 0.0001;
        assert_eq!(value.to_bits(), 0x3F1A_36E2_EB1C_432D_u64);
        let expr = Expr::Float(PositiveFiniteF64::try_from(value).unwrap());
        assert_eq!(translate(&expr), "0.0001");
    }

    #[test]
    fn float_neg_exp_max_with_exponent() {
        let value: f64 = 0.00009999999999999999;
        assert_eq!(value.to_bits(), 0x3F1A_36E2_EB1C_432C_u64);
        let expr = Expr::Float(PositiveFiniteF64::try_from(value).unwrap());
        assert_eq!(translate(&expr), "9.999999999999999e-5");
    }

    #[test]
    fn float_max() {
        let expr = Expr::Float(PositiveFiniteF64::try_from(f64::MAX).unwrap());
        assert_eq!(translate(&expr), "1.7976931348623157e308");
    }

    #[test]
    fn float_min() {
        let expr = Expr::Float(PositiveFiniteF64::try_from(f64::MIN_POSITIVE).unwrap());
        assert_eq!(translate(&expr), "2.2250738585072014e-308");
    }

    #[test]
    fn float_max_subnormal() {
        // https://en.wikipedia.org/wiki/Double-precision_floating-point_format#Double-precision_examples
        // (Max. subnormal double)
        let value: f64 = 2.225073858507201e-308;
        assert_eq!(value.to_bits(), 0x000F_FFFF_FFFF_FFFF_u64);
        let expr = Expr::Float(PositiveFiniteF64::try_from(value).unwrap());
        assert_eq!(translate(&expr), "2.225073858507201e-308");
    }

    #[test]
    fn float_min_subnormal() {
        // https://en.wikipedia.org/wiki/Double-precision_floating-point_format#Double-precision_examples
        // (Min. subnormal positive double)
        let value: f64 = 5e-324;
        assert_eq!(value.to_bits(), 0x0000_0000_0000_0001_u64);
        let expr = Expr::Float(PositiveFiniteF64::try_from(value).unwrap());
        assert_eq!(translate(&expr), "5e-324");
    }

    #[test]
    fn str_empty() {
        let expr = Expr::Str(r"".to_string());
        assert_eq!(translate(&expr), r"''");
        // assert_eq!(translate(&expr), r#""""#);
    }

    #[test]
    fn str_with_backslash() {
        let expr = Expr::Str(r"w\x".to_string());
        // See https://doc.kaitai.io/user_guide.html#_basic_data_types:
        // > Single quoted strings are interpreted literally, i.e. backslash \, double quotes " and
        // > other possible special symbols carry no special meaning, they would be just considered
        // > a part of the string.
        assert_eq!(translate(&expr), r"'w\x'");
        // assert_eq!(translate(&expr), r#""w\\x""#);
    }

    #[test]
    fn str_with_double_quote() {
        let expr = Expr::Str(r#"y"z"#.to_string());
        // See https://doc.kaitai.io/user_guide.html#_basic_data_types:
        // > Single quoted strings are interpreted literally, i.e. backslash \, double quotes " and
        // > other possible special symbols carry no special meaning, they would be just considered
        // > a part of the string.
        assert_eq!(translate(&expr), r#"'y"z'"#);
        // assert_eq!(translate(&expr), r#""y\"z""#);
    }

    #[test]
    #[should_panic(expected = "strings containing a single quote (') not supported yet")]
    fn str_with_single_quote() {
        let expr = Expr::Str(r"a'b".to_string());
        // See https://doc.kaitai.io/user_guide.html#_basic_data_types:
        // > Everything between single quotes is interpreted literally, i.e. there is no way one can
        // > include a single quote inside a single quoted string.
        translate(&expr);
        // assert_eq!(translate(&expr), r#""a'b""#);
    }

    #[test]
    fn bool_false() {
        let expr = Expr::Bool(false);
        assert_eq!(translate(&expr), "false");
    }

    #[test]
    fn bool_true() {
        let expr = Expr::Bool(true);
        assert_eq!(translate(&expr), "true");
    }

    #[test]
    fn enum_member() {
        let expr = Expr::EnumMember {
            enum_path: vec!["some_type", "port"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            label: "http".to_string(),
        };
        assert_eq!(translate(&expr), "some_type::port::http");
    }

    #[test]
    fn list() {
        let expr = Expr::List(vec![
            Expr::Str("literal".to_string()),
            Expr::Name("my_string_attr".to_string()),
            Expr::BinaryOp {
                l: Box::new(Expr::Str("hello ".to_string())),
                op: BinaryOp::Add,
                r: Box::new(Expr::Name("person_name".to_string())),
            },
        ]);
        assert_eq!(
            translate(&expr),
            "['literal', my_string_attr, ('hello ' + person_name)]"
        );
    }

    #[test]
    fn name() {
        let expr = Expr::Name("note_len".to_string());
        assert_eq!(translate(&expr), "note_len");
    }

    #[test]
    fn name_parent() {
        let expr = Expr::Name("_parent".to_string());
        assert_eq!(translate(&expr), "_parent");
    }

    #[test]
    fn attribute_zero_int_to_s() {
        let expr = Expr::Attribute {
            value: Box::new(Expr::Int(BigUint::from(0_u32))),
            attr_name: "to_s".to_string(),
        };
        assert_eq!(translate(&expr), "0.to_s");
    }

    #[test]
    fn attribute_neg_int_to_s() {
        let expr = Expr::Attribute {
            value: Box::new(Expr::UnaryOp {
                op: UnaryOp::Neg,
                v: Box::new(Expr::Int(BigUint::from(3_u32))),
            }),
            attr_name: "to_s".to_string(),
        };
        assert_eq!(translate(&expr), "(-3).to_s");
    }

    #[test]
    fn attribute_pos_float_to_i() {
        let expr = Expr::Attribute {
            value: Box::new(Expr::Float(PositiveFiniteF64::try_from(1.75).unwrap())),
            attr_name: "to_i".to_string(),
        };
        assert_eq!(translate(&expr), "1.75.to_i");
    }

    #[test]
    fn attribute_neg_float_to_i() {
        let expr = Expr::Attribute {
            value: Box::new(Expr::UnaryOp {
                op: UnaryOp::Neg,
                v: Box::new(Expr::Float(PositiveFiniteF64::try_from(1.75).unwrap())),
            }),
            attr_name: "to_i".to_string(),
        };
        assert_eq!(translate(&expr), "(-1.75).to_i");
    }

    #[test]
    fn attribute_enum_member_to_i() {
        let expr = Expr::Attribute {
            value: Box::new(Expr::EnumMember {
                enum_path: vec!["record_types".to_string()],
                label: "uint64".to_string(),
            }),
            attr_name: "to_i".to_string(),
        };
        assert_eq!(translate(&expr), "record_types::uint64.to_i");
    }

    #[test]
    fn method_call() {
        let expr = Expr::MethodCall {
            value: Box::new(Expr::BinaryOp {
                l: Box::new(Expr::Name("str_0_to_4".to_string())),
                op: BinaryOp::Add,
                r: Box::new(Expr::Str("56789".to_string())),
            }),
            method_name: "substring".to_string(),
            args: vec![
                Expr::Int(BigUint::from(2_u32)),
                Expr::Int(BigUint::from(7_u32)),
            ],
        };
        assert_eq!(translate(&expr), "(str_0_to_4 + '56789').substring(2, 7)");
    }

    #[test]
    fn unary_neg() {
        let expr = Expr::UnaryOp {
            op: UnaryOp::Neg,
            v: Box::new(Expr::Int(BigUint::from(100_u32))),
        };
        assert_eq!(translate(&expr), "(-100)");
    }

    #[test]
    fn unary_not() {
        let expr = Expr::UnaryOp {
            op: UnaryOp::Not,
            v: Box::new(Expr::Bool(false)),
        };
        assert_eq!(translate(&expr), "(not false)");
    }

    #[test]
    fn unary_inv() {
        let expr = Expr::UnaryOp {
            op: UnaryOp::Inv,
            v: Box::new(Expr::Int(BigUint::from(3_u32))),
        };
        assert_eq!(translate(&expr), "(~3)");
    }

    #[test]
    fn binary_add() {
        let expr = Expr::BinaryOp {
            l: Box::new(Expr::Str("Hello ".to_string())),
            op: BinaryOp::Add,
            r: Box::new(Expr::Str("world!".to_string())),
        };
        assert_eq!(translate(&expr), "('Hello ' + 'world!')");
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn binary_sub() {
        let expr = Expr::BinaryOp {
            l: Box::new(Expr::Float(PositiveFiniteF64::try_from(6.28).unwrap())),
            op: BinaryOp::Sub,
            r: Box::new(Expr::UnaryOp {
                op: UnaryOp::Neg,
                v: Box::new(Expr::Float(PositiveFiniteF64::try_from(2.72).unwrap())),
            }),
        };
        assert_eq!(translate(&expr), "(6.28 - (-2.72))");
    }

    #[test]
    fn binary_mul() {
        let expr = Expr::BinaryOp {
            l: Box::new(Expr::Int(BigUint::from(2_u32))),
            op: BinaryOp::Mul,
            r: Box::new(Expr::UnaryOp {
                op: UnaryOp::Neg,
                v: Box::new(Expr::Int(BigUint::from(3_u32))),
            }),
        };
        assert_eq!(translate(&expr), "(2 * (-3))");
    }

    #[test]
    fn binary_div() {
        let expr = Expr::BinaryOp {
            l: Box::new(Expr::Float(PositiveFiniteF64::try_from(64.5).unwrap())),
            op: BinaryOp::Div,
            r: Box::new(Expr::Int(BigUint::from(100_u32))),
        };
        assert_eq!(translate(&expr), "(64.5 / 100)");
    }

    #[test]
    fn binary_rem() {
        let expr = Expr::BinaryOp {
            l: Box::new(Expr::UnaryOp {
                op: UnaryOp::Neg,
                v: Box::new(Expr::Int(BigUint::from(3_u32))),
            }),
            op: BinaryOp::Rem,
            r: Box::new(Expr::Int(BigUint::from(4_u32))),
        };
        assert_eq!(translate(&expr), "((-3) % 4)");
    }

    #[test]
    fn binary_eq() {
        let expr = Expr::BinaryOp {
            l: Box::new(Expr::Bool(false)),
            op: BinaryOp::Eq,
            r: Box::new(Expr::CondOp {
                cond: Box::new(Expr::Bool(true)),
                if_true: Box::new(Expr::Attribute {
                    value: Box::new(Expr::Name("_io".to_string())),
                    attr_name: "eof".to_string(),
                }),
                if_false: Box::new(Expr::Bool(false)),
            }),
        };
        assert_eq!(translate(&expr), "(false == (true ? _io.eof : false))");
    }

    #[test]
    fn binary_ne() {
        let expr = Expr::BinaryOp {
            l: Box::new(Expr::Bool(true)),
            op: BinaryOp::Ne,
            r: Box::new(Expr::CondOp {
                cond: Box::new(Expr::Bool(true)),
                if_true: Box::new(Expr::Attribute {
                    value: Box::new(Expr::Name("_io".to_string())),
                    attr_name: "eof".to_string(),
                }),
                if_false: Box::new(Expr::Bool(false)),
            }),
        };
        assert_eq!(translate(&expr), "(true != (true ? _io.eof : false))");
    }

    #[test]
    fn binary_lt() {
        let expr = Expr::BinaryOp {
            l: Box::new(Expr::Float(PositiveFiniteF64::try_from(0.3).unwrap())),
            op: BinaryOp::Lt,
            r: Box::new(Expr::BinaryOp {
                l: Box::new(Expr::Float(PositiveFiniteF64::try_from(0.1).unwrap())),
                op: BinaryOp::Add,
                r: Box::new(Expr::Float(PositiveFiniteF64::try_from(0.2).unwrap())),
            }),
        };
        assert_eq!(translate(&expr), "(0.3 < (0.1 + 0.2))");
    }

    #[test]
    fn binary_gt() {
        let expr = Expr::BinaryOp {
            l: Box::new(Expr::BinaryOp {
                l: Box::new(Expr::Float(PositiveFiniteF64::try_from(0.1).unwrap())),
                op: BinaryOp::Add,
                r: Box::new(Expr::Float(PositiveFiniteF64::try_from(0.2).unwrap())),
            }),
            op: BinaryOp::Gt,
            r: Box::new(Expr::Float(PositiveFiniteF64::try_from(0.3).unwrap())),
        };
        assert_eq!(translate(&expr), "((0.1 + 0.2) > 0.3)");
    }

    #[test]
    fn binary_le() {
        let expr = Expr::BinaryOp {
            l: Box::new(Expr::BinaryOp {
                l: Box::new(Expr::Float(PositiveFiniteF64::try_from(0.1).unwrap())),
                op: BinaryOp::Add,
                r: Box::new(Expr::Float(PositiveFiniteF64::try_from(0.2).unwrap())),
            }),
            op: BinaryOp::Le,
            r: Box::new(Expr::Float(PositiveFiniteF64::try_from(0.3).unwrap())),
        };
        assert_eq!(translate(&expr), "((0.1 + 0.2) <= 0.3)");
    }

    #[test]
    fn binary_ge() {
        let expr = Expr::BinaryOp {
            l: Box::new(Expr::Float(PositiveFiniteF64::try_from(0.3).unwrap())),
            op: BinaryOp::Ge,
            r: Box::new(Expr::BinaryOp {
                l: Box::new(Expr::Float(PositiveFiniteF64::try_from(0.1).unwrap())),
                op: BinaryOp::Add,
                r: Box::new(Expr::Float(PositiveFiniteF64::try_from(0.2).unwrap())),
            }),
        };
        assert_eq!(translate(&expr), "(0.3 >= (0.1 + 0.2))");
    }

    #[test]
    fn binary_and() {
        let expr = Expr::BinaryOp {
            l: Box::new(Expr::UnaryOp {
                op: UnaryOp::Not,
                v: Box::new(Expr::Bool(true)),
            }),
            op: BinaryOp::And,
            r: Box::new(Expr::Bool(false)),
        };
        assert_eq!(translate(&expr), "((not true) and false)");
    }

    #[test]
    fn binary_or() {
        let expr = Expr::BinaryOp {
            l: Box::new(Expr::UnaryOp {
                op: UnaryOp::Not,
                v: Box::new(Expr::Bool(false)),
            }),
            op: BinaryOp::Or,
            r: Box::new(Expr::Bool(true)),
        };
        assert_eq!(translate(&expr), "((not false) or true)");
    }

    #[test]
    fn binary_bit_or() {
        let expr = Expr::BinaryOp {
            l: Box::new(Expr::Name("lo".to_string())),
            op: BinaryOp::BitOr,
            r: Box::new(Expr::BinaryOp {
                l: Box::new(Expr::Name("hi".to_string())),
                op: BinaryOp::Shl,
                r: Box::new(Expr::Int(BigUint::from(16_u32))),
            }),
        };
        assert_eq!(translate(&expr), "(lo | (hi << 16))");
    }

    #[test]
    fn binary_bit_xor() {
        let expr = Expr::BinaryOp {
            l: Box::new(Expr::BinaryOp {
                l: Box::new(Expr::Name("x".to_string())),
                op: BinaryOp::BitXor,
                r: Box::new(Expr::Name("y".to_string())),
            }),
            op: BinaryOp::Lt,
            r: Box::new(Expr::Int(BigUint::from(0_u32))),
        };
        assert_eq!(translate(&expr), "((x ^ y) < 0)");
    }

    #[test]
    fn binary_bit_and() {
        let expr = Expr::BinaryOp {
            l: Box::new(Expr::BinaryOp {
                l: Box::new(Expr::Attribute {
                    value: Box::new(Expr::Name("_io".to_string())),
                    attr_name: "pos".to_string(),
                }),
                op: BinaryOp::Add,
                r: Box::new(Expr::Int(BigUint::from(3_u32))),
            }),
            op: BinaryOp::BitAnd,
            r: Box::new(Expr::UnaryOp {
                op: UnaryOp::Inv,
                v: Box::new(Expr::Int(BigUint::from(3_u32))),
            }),
        };
        assert_eq!(translate(&expr), "((_io.pos + 3) & (~3))");
    }

    #[test]
    fn binary_shl() {
        let expr = Expr::BinaryOp {
            l: Box::new(Expr::UnaryOp {
                op: UnaryOp::Neg,
                v: Box::new(Expr::Int(BigUint::from(1_u32))),
            }),
            op: BinaryOp::Shl,
            r: Box::new(Expr::Int(BigUint::from(3_u32))),
        };
        assert_eq!(translate(&expr), "((-1) << 3)");
    }

    #[test]
    fn binary_shr() {
        let expr = Expr::BinaryOp {
            l: Box::new(Expr::BinaryOp {
                l: Box::new(Expr::Name("packed".to_string())),
                op: BinaryOp::BitAnd,
                r: Box::new(Expr::Int(BigUint::from(0b1111_1000_0000_0000_u32))),
            }),
            op: BinaryOp::Shr,
            r: Box::new(Expr::BinaryOp {
                l: Box::new(Expr::Int(BigUint::from(3_u32))),
                op: BinaryOp::Add,
                r: Box::new(Expr::Int(BigUint::from(8_u32))),
            }),
        };
        assert_eq!(translate(&expr), "((packed & 63488) >> (3 + 8))");
    }

    #[test]
    fn cond_op() {
        let expr = Expr::CondOp {
            cond: Box::new(Expr::BinaryOp {
                l: Box::new(Expr::Bool(true)),
                op: BinaryOp::Eq,
                r: Box::new(Expr::Bool(false)),
            }),
            if_true: Box::new(Expr::Str("nonsense".to_string())),
            if_false: Box::new(Expr::Str("makes sense".to_string())),
        };
        assert_eq!(
            translate(&expr),
            "((true == false) ? 'nonsense' : 'makes sense')"
        )
    }

    #[test]
    fn subscript_attr() {
        let expr = Expr::Subscript {
            value: Box::new(Expr::Attribute {
                value: Box::new(Expr::Name("cont".to_string())),
                attr_name: "items".to_string(),
            }),
            idx: Box::new(Expr::Int(BigUint::from(0_u32))),
        };
        assert_eq!(translate(&expr), "cont.items[0]");
    }

    #[test]
    fn subscript_nested() {
        let expr = Expr::Subscript {
            value: Box::new(Expr::Subscript {
                value: Box::new(Expr::List(vec![
                    Expr::List(vec![
                        Expr::Int(BigUint::from(1_u32)),
                        Expr::Int(BigUint::from(300_u32)),
                    ]),
                    Expr::List(vec![
                        Expr::UnaryOp {
                            op: UnaryOp::Neg,
                            v: Box::new(Expr::Int(BigUint::from(1_u32))),
                        },
                        Expr::Int(BigUint::from(1_u32)),
                    ]),
                ])),
                idx: Box::new(Expr::Attribute {
                    value: Box::new(Expr::Str("1".to_string())),
                    attr_name: "to_i".to_string(),
                }),
            }),
            idx: Box::new(Expr::Int(BigUint::from(0_u32))),
        };
        assert_eq!(translate(&expr), "[[1, 300], [(-1), 1]]['1'.to_i][0]");
    }
}
