//! Builtin constants provided by the PostScript execution environment

use super::{
    object::{PostScriptArray, PostScriptDictionary, PostScriptObject, PostScriptString},
    operator::PostscriptOperator,
    interpreter::PostscriptInterpreter,
};

pub(super) fn gen_system_dict() -> PostScriptDictionary {
    let mut system_dict = PostScriptDictionary::new();

    system_dict.insert(
        PostScriptString::from_bytes(b"abs".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::Abs),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"add".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::Add),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"dict".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::Dict),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"begin".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::Begin),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"dup".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::Dup),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"def".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::Def),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"readonly".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::ReadOnly),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"executeonly".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::ExecuteOnly),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"noaccess".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::NoAccess),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"false".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::False),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"true".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::True),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"end".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::End),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"currentfile".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::CurrentFile),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"eexec".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::EExec),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"arraystart".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::ArrayStart),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"arrayend".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::ArrayEnd),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"{".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::ProcedureStart),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"}".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::ProcedureEnd),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"currentdict".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::CurrentDict),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"string".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::String),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"exch".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::Exch),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"readstring".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::ReadString),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"pop".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::Pop),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"put".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::Put),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"internaldict".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::InternalDict),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"cvx".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::Cvx),
    );
    system_dict.insert(
        PostScriptString::from_bytes(b"maxlength".to_vec()),
        PostScriptObject::Operator(PostscriptOperator::MaxLength),
    );

    system_dict
}

#[rustfmt::skip]
pub(super) static STANDARD_ENCODING: &[Option<&str>] = &[
    /*\00x*/ None, None, None, None, None, None, None, None,
    /*\01x*/ None, None, None, None, None, None, None, None,
    /*\02x*/ None, None, None, None, None, None, None, None,
    /*\03x*/ None, None, None, None, None, None, None, None,
    /*\04x*/ Some("space"), Some("exclam"), Some("quotedbl"), Some("numbersign"),
             Some("dollar"), Some("percent"), Some("ampersand"), Some("quoteright"),
    /*\05x*/ Some("parenleft"), Some("parenright"), Some("asterisk"), Some("plus"),
             Some("comma"), Some("hyphen"), Some("period"), Some("slash"),
    /*\06x*/ Some("zero"), Some("one"), Some("two"), Some("three"),
             Some("four"), Some("five"), Some("six"), Some("seven"),
    /*\07x*/ Some("eight"), Some("nine"), Some("colon"), Some("semicolon"),
             Some("less"), Some("equal"), Some("greater"), Some("question"),
    /*\10x*/ Some("at"), Some("A"), Some("B"), Some("C"),
             Some("D"), Some("E"), Some("F"), Some("G"),
    /*\11x*/ Some("H"), Some("I"), Some("J"), Some("K"),
             Some("L"), Some("M"), Some("N"), Some("O"),
    /*\12x*/ Some("P"), Some("Q"), Some("R"), Some("S"),
             Some("T"), Some("U"), Some("V"), Some("W"),
    /*\13x*/ Some("X"), Some("Y"), Some("Z"), Some("bracketleft"),
             Some("backslash"), Some("bracketright"), Some("asciicircum"), Some("underscore"),
    /*\14x*/ Some("quoteleft"), Some("a"), Some("b"), Some("c"),
             Some("d"), Some("e"), Some("f"), Some("g"),
    /*\15x*/ Some("h"), Some("i"), Some("j"), Some("k"),
             Some("l"), Some("m"), Some("n"), Some("o"),
    /*\16x*/ Some("p"), Some("q"), Some("r"), Some("s"),
             Some("t"), Some("u"), Some("v"), Some("w"),
    /*\17x*/ Some("x"), Some("y"), Some("z"), Some("braceleft"),
             Some("bar"), Some("braceright"), Some("asciitilde"), None,
    /*\20x*/ None, None, None, None, None, None, None, None,
    /*\21x*/ None, None, None, None, None, None, None, None,
    /*\22x*/ None, None, None, None, None, None, None, None,
    /*\23x*/ None, None, None, None, None, None, None, None,
    /*\24x*/ None, Some("exclamdown"), Some("cent"), Some("sterling"),
             Some("fraction"), Some("yen"), Some("florin"), Some("section"),
    /*\25x*/ Some("currency"), Some("quotesingle"), Some("quotedblleft"), Some("guillemotleft"),
             Some("guilsinglleft"), Some("guilsinglright"), Some("fi"), Some("fl"),
    /*\26x*/ None, Some("endash"), Some("dagger"), Some("daggerdbl"),
             Some("periodcentered"), None, Some("paragraph"), Some("bullet"),
    /*\27x*/ Some("quotesinglbase"), Some("quotedblbase"), Some("quotedblright"), Some("guillemotright"),
             Some("ellipsis"), Some("perthousand"), None, Some("questiondown"),
    /*\30x*/ None, Some("grave"), Some("acute"), Some("circumflex"),
             Some("tilde"), Some("macron2"), Some("breve"), Some("dotaccent"),
    /*\31x*/ Some("dieresis"), None, Some("ring"), Some("cedilla"),
             None, Some("hungarumlaut"), Some("ogonek"), Some("caron"),
    /*\32x*/ Some("emdash"), None, None, None, None, None, None, None,
    /*\33x*/ None, None, None, None, None, None, None, None,
    /*\34x*/ None, Some("AE"), None, Some("ordfeminine"), None, None, None, None,
    /*\35x*/ Some("Lslash"), Some("Oslash"), Some("oe"), Some("ordmasculine"), None, None, None, None,
    /*\36x*/ None, Some("ae"), None, None, None, Some("dotlessi"), None, None,
    /*\37x*/ Some("lslash"), Some("oslash"), Some("OE"), Some("germandbls"), None, None, None, None,
];

pub(super) fn gen_standard_encoding_vector(
    interpreter: &mut PostscriptInterpreter,
) -> PostScriptArray {
    PostScriptArray::from_objects(
        STANDARD_ENCODING
            .iter()
            .map(|name| match name {
                &Some(s) => interpreter
                    .intern_string(PostScriptString::from_bytes(s.to_owned().into_bytes())),
                None => PostScriptObject::Null,
            })
            .collect(),
    )
}
