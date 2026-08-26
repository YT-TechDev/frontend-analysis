//! Production-side TC-S5 verification.
//!
//! The complete P1-P21 production matrix is filled after the production
//! result/session boundary compiles. This module never imports the
//! candidate-independent validation machine; production observations are
//! checked only against hand-authored expected values.

use super::driver::construct_document_shell;
use super::super::tokenizer::resource::HtmlTokenizerLimits;
use crate::{SourceId, SourceText};

fn limits() -> HtmlTokenizerLimits {
    HtmlTokenizerLimits::new(1_024, 8_192, 1_024, 1_024, 256, 4_096, 1_024)
}

#[test]
fn tc_s5_production_smoke_reaches_paragraph_candidate() {
    let source = SourceText::new(SourceId::new(1), "<body><p>x</p>".to_owned());
    let result = construct_document_shell(&source, limits()).expect("TC-S5 production smoke");
    assert!(result.is_complete());
}
