# HTML Analysis Parser: First Capability

## Purpose and Authority

This document describes the first source-backed HTML analysis-parser
capability implemented under Issue #115, built on the architecture approved
in Issue #114 and [ADR 0007](../decisions/0007-own-lossless-source-parsers.md).
It specializes [Source Parser Ownership](../architecture/SOURCE_PARSER_OWNERSHIP.md)
and does not redefine tokenizer contracts owned by
[HTML Tokenizer Validation](HTML_TOKENIZER_VALIDATION.md). It does not claim
complete HTML parsing, tree construction, or DOM behavior.

## Capability

The first capability answers exactly:

> Which explicit start-tag occurrences are recognized in the retained source,
> and what exact raw half-open ranges identify each complete authored tag and
> raw tag-name spelling?

This is a capability-specific authored occurrence projection, not a generic
parser-event stream, syntax tree, or universal AST.

## Input Requirement

The parser (`crates/frontend-analysis-core/src/html/parser/mod.rs`) consumes
one already-validated `HtmlTokenizerRunResult` by value. It does not accept
raw `SourceText` and does not invoke the tokenizer itself: connecting
`SourceText` to tokenization and parsing as one callable operation is owned by
the following Core-integration Leaf, not by this capability.

## Occurrence Semantics

For every `HtmlTagKind::Start` token, the parser projects one
`HtmlExplicitStartTagOccurrence` carrying:

- `origin_token_index`: the index of the originating token in the retained
  tokenizer run, for internal provenance and traceability only, not a stable
  external identifier;
- `complete`: the complete authored start-tag `SourceAnchor`, cloned from the
  originating token's tag-complete evidence;
- `raw_name`: the exact raw tag-name `SourceAnchor`, cloned from the
  originating token's name evidence, never the interpreted (e.g. lowercased)
  spelling.

Semantic authored occurrence identity is the retained source identity plus
the complete authored range. Occurrences are emitted in deterministic
tokenizer/source order, and duplicate raw spellings at different offsets
remain distinct occurrences.

`HtmlTagKind::End` tokens and character-data tokens are consumed as validated
input but are not projected by this capability; they are not thereby
classified unsupported. EOF is termination evidence only.

## Source Evidence

Every occurrence's `complete` and `raw_name` anchors are cloned directly from
already-validated tokenizer evidence (`HtmlTagToken::complete()` and
`HtmlTagToken::name().source()`). The parser performs no source rescan,
delimiter scan, decoded-length inference, endpoint reconstruction, or token
replay.

The resulting `HtmlExplicitStartTagAnalysis` validates, before becoming
trusted, that every occurrence's origin token index is in bounds and
references an `HtmlToken::Tag` of kind `Start`; that the occurrence's
`complete` and `raw_name` anchors exactly match that token's own evidence;
that occurrences appear in strictly increasing origin-token order; and that
the occurrence vector exactly covers every start-tag token in the retained
run, with no token missing and no extra occurrence. A violated relationship
returns a deterministic, owned `HtmlAnalysisParserContractError` rather than
panicking or silently dropping evidence. This failure channel is distinct
from tokenizer diagnostics.

## Completeness Propagation

`HtmlExplicitStartTagAnalysis` retains the consumed `HtmlTokenizerRunResult`
by value rather than re-encoding a parser-specific completion, diagnostics,
coverage, or resource duplicate. Tokenizer completion, diagnostics, coverage,
and resource evidence remain authoritative through
`HtmlExplicitStartTagAnalysis::tokenizer_run()`.

Parser completeness is therefore monotonic with, and can never exceed,
tokenizer completeness: an incomplete tokenizer run (unsupported capability,
resource limit, invalid configuration, or internal invariant failure) always
produces an analysis whose retained tokenizer run remains incomplete, even
when zero start-tag occurrences were projected. Zero occurrences from an
incomplete run is never represented as clean absence or clean success.

## Deferred: Matching, Nesting, and Tree Construction

This capability does not build an open-element stack, match start and end
tags, infer parent/child relationships or nesting depth, or synthesize
implied structure from token order. Those responsibilities belong to a later,
separately approved tree-construction capability. Authored occurrences here
are authored-syntax evidence only and must never be confused with future
synthesized structure.

## Deferred: Attributes and Self-Closing Evidence

The tokenizer already retains authenticated attribute and self-closing
evidence on each `HtmlTagToken`. This first capability does not project that
evidence into the occurrence contract, because it is not required to answer
the approved first question. No evidence is discarded: it remains available
through the retained tokenizer run and may be projected by a later focused
capability when a named analysis consumer requires it.

## Visibility

`HtmlExplicitStartTagOccurrence`, `HtmlExplicitStartTagAnalysis`,
`HtmlAnalysisParserContractError`, and `analyze_explicit_start_tags` are all
`pub(crate)`. No public parser API, serialization contract, or SemVer
commitment is introduced by this capability.

## Resource and WASM Implications

The parser is synchronous, single-owner, iterative, and non-recursive: one
pass over the retained tokenizer token slice, with no source rescan, no
second tokenizer, and no external parser dependency. Expected cost is linear
in token count, with storage proportional to the number of projected
start-tag occurrences. No new parser-specific resource taxonomy is
introduced: the tokenizer's own bounded execution and retained result remain
the authoritative resource evidence for this slice. The implementation
introduces no native-only assumption that would prevent equivalent analysis
meaning under a future approved WASM target.

## Extension Boundary

Future comment, doctype, character-reference, raw-text, foreign-content,
end-tag-analysis, attribute-projection, and tree-construction capabilities
must extend this or a sibling capability-specific model deliberately. They
must not require breaking retained-source provenance, must not reinterpret
existing authored occurrences as synthesized structure, and must not
introduce a generic shared parser/event abstraction across HTML, CSS, and
ECMAScript without a demonstrated cross-language invariant.

## Candidate-Independent Validation

Production parser output is not its own oracle. Expected occurrence
projections are derived independently by filtering the existing 76
candidate-independent tokenizer fixtures (`HTML_TOKENIZER_VALIDATION.md`'s
72-fixture initial corpus plus its 4-fixture supplemental regression corpus)
for start-tag tokens, in
`crates/frontend-analysis-core/src/html/tokenizer/validation/parser_gate.rs`.
This module lives beside the tokenizer's own candidate-independent tests so
it can reuse the existing private fixture/gold types without widening their
visibility or duplicating the 76-fixture corpus; it does not add, remove, or
modify any tokenizer fixture.

## Core Integration Boundary

This document does not define a callable Core operation that accepts raw
`SourceText`. Connecting `SourceText` through tokenization and this parsing
capability into one Core-facing analysis operation is owned by the
integration Leaf that follows #115.
