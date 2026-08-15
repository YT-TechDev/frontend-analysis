//! Frozen first-envelope validation inventory for Issue #197.
//!
//! The 37 entries below are audited Early Error *clause containers*, not rule
//! counts. Rule units preserve both active first-envelope obligations and
//! normative rules whose applicability was audited and found inactive under
//! the frozen Script / host-neutral qualification envelope.

use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ContainerFamily {
    Lexical,
    Expression,
    StatementDeclaration,
    FunctionClass,
    Script,
    RegExpPattern,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EarlyErrorContainer {
    pub(crate) id: &'static str,
    pub(crate) family: ContainerFamily,
    pub(crate) title: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuleUnitKind {
    /// A normative Early Error rule active under the frozen first envelope.
    NormativeRule,
    /// A normative rule whose applicability was audited and found inactive
    /// under the frozen first envelope (for example due to Script goal or
    /// the empty Normative Optional web-source feature set).
    EnvelopeInactiveRule,
    /// A fail-closed marker for a container whose rule expansion is not yet
    /// complete. No sentinel may remain at rule-expansion readiness.
    ExpansionSentinel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RuleUnit {
    pub(crate) id: &'static str,
    pub(crate) container_id: &'static str,
    pub(crate) kind: RuleUnitKind,
    pub(crate) normative_locator: &'static str,
    pub(crate) dependencies: &'static [&'static str],
    pub(crate) gold_obligations: &'static [&'static str],
    pub(crate) gold_fixture_ids: &'static [&'static str],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum InventoryError {
    DuplicateContainer(&'static str),
    MissingContainer(&'static str),
    UnknownContainerForRule(&'static str),
    MissingRuleUnitForContainer(&'static str),
    DuplicateRuleUnit(&'static str),
    EmptyNormativeLocator(&'static str),
    MissingGoldObligations(&'static str),
}

pub(crate) const CONTAINERS: [EarlyErrorContainer; 37] = [
    container("EE-01", ContainerFamily::Lexical, "Identifier Names"),
    container("EE-02", ContainerFamily::Lexical, "Numeric Literals"),
    container("EE-03", ContainerFamily::Lexical, "String Literals"),
    container("EE-04", ContainerFamily::Expression, "Identifiers"),
    container("EE-05", ContainerFamily::Expression, "Object Initializer"),
    container(
        "EE-06",
        ContainerFamily::Expression,
        "Regular Expression Literals",
    ),
    container("EE-07", ContainerFamily::Expression, "Template Literals"),
    container("EE-08", ContainerFamily::Expression, "Grouping Operator"),
    container(
        "EE-09",
        ContainerFamily::Expression,
        "Left-Hand-Side Expressions",
    ),
    container("EE-10", ContainerFamily::Expression, "Update Expressions"),
    container("EE-11", ContainerFamily::Expression, "delete Operator"),
    container("EE-12", ContainerFamily::Expression, "Assignment Operators"),
    container(
        "EE-13",
        ContainerFamily::Expression,
        "Destructuring Assignment",
    ),
    container("EE-14", ContainerFamily::StatementDeclaration, "Block"),
    container(
        "EE-15",
        ContainerFamily::StatementDeclaration,
        "let / const Declarations",
    ),
    container(
        "EE-16",
        ContainerFamily::StatementDeclaration,
        "if Statement",
    ),
    container(
        "EE-17",
        ContainerFamily::StatementDeclaration,
        "do / while Statement",
    ),
    container(
        "EE-18",
        ContainerFamily::StatementDeclaration,
        "while Statement",
    ),
    container(
        "EE-19",
        ContainerFamily::StatementDeclaration,
        "for Statement",
    ),
    container(
        "EE-20",
        ContainerFamily::StatementDeclaration,
        "for-in / for-of Statement",
    ),
    container(
        "EE-21",
        ContainerFamily::StatementDeclaration,
        "continue Statement",
    ),
    container(
        "EE-22",
        ContainerFamily::StatementDeclaration,
        "break Statement",
    ),
    container(
        "EE-23",
        ContainerFamily::StatementDeclaration,
        "with Statement",
    ),
    container(
        "EE-24",
        ContainerFamily::StatementDeclaration,
        "switch Statement",
    ),
    container(
        "EE-25",
        ContainerFamily::StatementDeclaration,
        "Labelled Statements",
    ),
    container(
        "EE-26",
        ContainerFamily::StatementDeclaration,
        "try / catch",
    ),
    container("EE-27", ContainerFamily::FunctionClass, "Parameter Lists"),
    container(
        "EE-28",
        ContainerFamily::FunctionClass,
        "Function Definitions",
    ),
    container(
        "EE-29",
        ContainerFamily::FunctionClass,
        "Arrow Function Definitions",
    ),
    container(
        "EE-30",
        ContainerFamily::FunctionClass,
        "Method Definitions",
    ),
    container(
        "EE-31",
        ContainerFamily::FunctionClass,
        "Generator Function Definitions",
    ),
    container(
        "EE-32",
        ContainerFamily::FunctionClass,
        "Async Generator Function Definitions",
    ),
    container("EE-33", ContainerFamily::FunctionClass, "Class Definitions"),
    container(
        "EE-34",
        ContainerFamily::FunctionClass,
        "Async Function Definitions",
    ),
    container(
        "EE-35",
        ContainerFamily::FunctionClass,
        "Async Arrow Function Definitions",
    ),
    container("EE-36", ContainerFamily::Script, "Script"),
    container("EE-37", ContainerFamily::RegExpPattern, "RegExp Pattern"),
];

const fn container(
    id: &'static str,
    family: ContainerFamily,
    title: &'static str,
) -> EarlyErrorContainer {
    EarlyErrorContainer { id, family, title }
}

const ACTIVE_GOLD: &[&str] = &[
    "rejecting witness",
    "nearest qualifying or non-trigger witness",
    "applicability boundary witness when material",
    "source/provenance assertion when authored evidence is reported",
];
const INACTIVE_GOLD: &[&str] = &[
    "retain the audited envelope-inactivity rationale",
    "boundary witness preventing accidental activation when material",
];
const EXPANSION_REQUIRED: &[&str] = &["expand every audited production/rule condition"];

/// Complete audited rule-level inventory for the frozen first envelope.
///
/// Rule expansion completeness and executable gold coverage are deliberately
/// separate. A rule can be fully inventoried while its executable gold is
/// still pending; aggregate conformance must remain fail-closed in that state.
pub(crate) const RULE_UNITS: &[RuleUnit] = &[
    // EE-01
    active_rule(
        "EE-01-R01",
        "EE-01",
        "IdentifierStart :: \\\\ UnicodeEscapeSequence / escaped code point must match IdentifierStartChar",
        &["IdentifierCodePoint", "Unicode ID_Start"],
        &[],
    ),
    active_rule(
        "EE-01-R02",
        "EE-01",
        "IdentifierPart :: \\\\ UnicodeEscapeSequence / escaped code point must match IdentifierPartChar",
        &["IdentifierCodePoint", "Unicode ID_Continue"],
        &[],
    ),
    // EE-02
    active_rule(
        "EE-02-R01",
        "EE-02",
        "LegacyOctalIntegerLiteral | NonOctalDecimalIntegerLiteral / strict-mode rejection",
        &["strictness", "MV"],
        &[],
    ),
    // EE-03
    active_rule(
        "EE-03-R01",
        "EE-03",
        "LegacyOctalEscapeSequence | NonOctalDecimalEscapeSequence / strict-mode rejection",
        &["strictness", "SV"],
        &[],
    ),
    // EE-04
    active_rule(
        "EE-04-R01",
        "EE-04",
        "BindingIdentifier : Identifier / strict eval-or-arguments rejection",
        &["IsStrict", "StringValue"],
        &[],
    ),
    active_rule(
        "EE-04-R02",
        "EE-04",
        "IdentifierReference | BindingIdentifier | LabelIdentifier : yield / strict-mode rejection",
        &["IsStrict"],
        &[],
    ),
    inactive_rule(
        "EE-04-I01",
        "EE-04",
        "IdentifierReference | BindingIdentifier | LabelIdentifier : await / Module-goal-only rejection",
        &["syntactic grammar goal"],
    ),
    active_rule(
        "EE-04-R03",
        "EE-04",
        "BindingIdentifier[Yield,Await] : yield / [Yield] parameter rejection",
        &["grammar parameter Yield"],
        &[],
    ),
    active_rule(
        "EE-04-R04",
        "EE-04",
        "BindingIdentifier[Yield,Await] : await / [Await] parameter rejection",
        &["grammar parameter Await"],
        &[],
    ),
    active_rule(
        "EE-04-R05",
        "EE-04",
        "IdentifierReference | BindingIdentifier | LabelIdentifier : Identifier / [Yield] + StringValue yield rejection",
        &["grammar parameter Yield", "StringValue"],
        &[],
    ),
    active_rule(
        "EE-04-R06",
        "EE-04",
        "IdentifierReference | BindingIdentifier | LabelIdentifier : Identifier / [Await] + StringValue await rejection",
        &["grammar parameter Await", "StringValue"],
        &[],
    ),
    active_rule(
        "EE-04-R07",
        "EE-04",
        "Identifier : IdentifierName but not ReservedWord / strict future-reserved-word rejection",
        &["IsStrict", "StringValue"],
        &[],
    ),
    inactive_rule(
        "EE-04-I02",
        "EE-04",
        "Identifier : IdentifierName but not ReservedWord / Module-goal await rejection",
        &["syntactic grammar goal", "StringValue"],
    ),
    active_rule(
        "EE-04-R08",
        "EE-04",
        "Identifier : IdentifierName but not ReservedWord / escaped ReservedWord StringValue rejection",
        &["StringValue"],
        &[],
    ),
    // EE-05
    active_rule(
        "EE-05-R01",
        "EE-05",
        "PropertyDefinition : MethodDefinition / HasDirectSuper rejection",
        &["HasDirectSuper"],
        &[],
    ),
    active_rule(
        "EE-05-R02",
        "EE-05",
        "PropertyDefinition : MethodDefinition / PrivateBoundIdentifiers must be empty",
        &["PrivateBoundIdentifiers"],
        &[],
    ),
    active_rule(
        "EE-05-R03",
        "EE-05",
        "PropertyDefinition : CoverInitializedName / source-text rejection outside cover reinterpretation",
        &["cover grammar applicability"],
        &[],
    ),
    active_rule(
        "EE-05-R04",
        "EE-05",
        "ObjectLiteral / duplicate colon-form __proto__ PropName rejection",
        &["PropertyNameList", "PropName"],
        &[],
    ),
    // EE-06
    active_rule(
        "EE-06-R01",
        "EE-06",
        "PrimaryExpression : RegularExpressionLiteral / IsValidRegularExpressionLiteral must be true",
        &["FlagText", "BodyText", "ParsePattern"],
        &["JS-GOLD-REGEXP-DUPFLAG-001"],
    ),
    // EE-07
    active_rule(
        "EE-07-R01",
        "EE-07",
        "NoSubstitutionTemplate / NotEscapeSequence in untagged template",
        &["Contains(NotEscapeSequence)", "Tagged"],
        &[],
    ),
    active_rule(
        "EE-07-R02",
        "EE-07",
        "TemplateStrings / cooked-string list length must be less than 2^32",
        &["TemplateStrings"],
        &[],
    ),
    active_rule(
        "EE-07-R03",
        "EE-07",
        "TemplateHead / NotEscapeSequence in untagged template",
        &["Contains(NotEscapeSequence)", "Tagged"],
        &[],
    ),
    active_rule(
        "EE-07-R04",
        "EE-07",
        "TemplateTail / NotEscapeSequence in untagged template",
        &["Contains(NotEscapeSequence)", "Tagged"],
        &[],
    ),
    active_rule(
        "EE-07-R05",
        "EE-07",
        "TemplateMiddle / NotEscapeSequence in untagged template",
        &["Contains(NotEscapeSequence)", "Tagged"],
        &[],
    ),
    // EE-08
    active_rule(
        "EE-08-R01",
        "EE-08",
        "PrimaryExpression : CoverParenthesizedExpressionAndArrowParameterList / must cover ParenthesizedExpression",
        &["cover grammar reinterpretation"],
        &[],
    ),
    // EE-09
    active_rule(
        "EE-09-R01",
        "EE-09",
        "OptionalChain template-literal productions / any matched source is rejected",
        &[],
        &[],
    ),
    active_rule(
        "EE-09-R02",
        "EE-09",
        "PrimaryExpression : import.meta / non-Module goal rejection",
        &["syntactic grammar goal"],
        &[],
    ),
    // EE-10
    active_rule(
        "EE-10-R01",
        "EE-10",
        "UpdateExpression : LeftHandSideExpression ++|-- / AssignmentTargetType must not be invalid",
        &["AssignmentTargetType"],
        &[],
    ),
    active_rule(
        "EE-10-R02",
        "EE-10",
        "UpdateExpression : ++|-- UnaryExpression / AssignmentTargetType must not be invalid",
        &["AssignmentTargetType"],
        &[],
    ),
    // EE-11
    active_rule(
        "EE-11-R01",
        "EE-11",
        "UnaryExpression : delete UnaryExpression / strict direct IdentifierReference or private target rejection",
        &["IsStrict", "cover recursion"],
        &[],
    ),
    active_rule(
        "EE-11-R02",
        "EE-11",
        "delete operand CoverParenthesizedExpressionAndArrowParameterList / recursive covered ParenthesizedExpression check",
        &["cover grammar reinterpretation", "IsStrict"],
        &[],
    ),
    // EE-12
    active_rule(
        "EE-12-R01",
        "EE-12",
        "AssignmentExpression : LeftHandSideExpression = AssignmentExpression / object-or-array target must cover AssignmentPattern",
        &["cover grammar reinterpretation"],
        &[],
    ),
    active_rule(
        "EE-12-R02",
        "EE-12",
        "AssignmentExpression : LeftHandSideExpression = AssignmentExpression / non-object-array AssignmentTargetType must not be invalid",
        &["AssignmentTargetType"],
        &[],
    ),
    active_rule(
        "EE-12-R03",
        "EE-12",
        "AssignmentExpression : LeftHandSideExpression AssignmentOperator AssignmentExpression / AssignmentTargetType must not be invalid",
        &["AssignmentTargetType"],
        &[],
    ),
    active_rule(
        "EE-12-R04",
        "EE-12",
        "AssignmentExpression : LeftHandSideExpression &&=||=??= AssignmentExpression / AssignmentTargetType must be simple",
        &["AssignmentTargetType"],
        &[],
    ),
    // EE-13
    active_rule(
        "EE-13-R01",
        "EE-13",
        "AssignmentProperty : IdentifierReference Initializer? / shorthand target AssignmentTargetType must be simple",
        &["AssignmentTargetType"],
        &[],
    ),
    active_rule(
        "EE-13-R02",
        "EE-13",
        "AssignmentRestProperty / target must not be ArrayLiteral or ObjectLiteral",
        &[],
        &[],
    ),
    active_rule(
        "EE-13-R03",
        "EE-13",
        "DestructuringAssignmentTarget : LeftHandSideExpression / ObjectLiteral|ArrayLiteral must cover AssignmentPattern",
        &["cover grammar reinterpretation"],
        &[],
    ),
    active_rule(
        "EE-13-R04",
        "EE-13",
        "DestructuringAssignmentTarget : LeftHandSideExpression / other AssignmentTargetType must be simple",
        &["AssignmentTargetType"],
        &[],
    ),
    // EE-14
    active_rule(
        "EE-14-R01",
        "EE-14",
        "Block : { StatementList } / duplicate LexicallyDeclaredNames",
        &["LexicallyDeclaredNames"],
        &[],
    ),
    active_rule(
        "EE-14-R02",
        "EE-14",
        "Block : { StatementList } / LexicallyDeclaredNames intersects VarDeclaredNames",
        &["LexicallyDeclaredNames", "VarDeclaredNames"],
        &[],
    ),
    // EE-15
    active_rule(
        "EE-15-R01",
        "EE-15",
        "LexicalDeclaration / BoundNames must not contain let",
        &["BoundNames"],
        &["JS-GOLD-LEXDECL-LET-BINDING-001"],
    ),
    active_rule(
        "EE-15-R02",
        "EE-15",
        "LexicalDeclaration / BoundNames must not contain duplicates",
        &["BoundNames"],
        &[],
    ),
    active_rule(
        "EE-15-R03",
        "EE-15",
        "LexicalBinding in const declaration / initializer required",
        &[],
        &[],
    ),
    // EE-16
    inactive_rule(
        "EE-16-I01",
        "EE-16",
        "IfStatement : if (Expression) Statement else Statement / first Statement IsLabelledFunction",
        &["IsLabelledFunction"],
    ),
    inactive_rule(
        "EE-16-I02",
        "EE-16",
        "IfStatement : if (Expression) Statement else Statement / second Statement IsLabelledFunction",
        &["IsLabelledFunction"],
    ),
    inactive_rule(
        "EE-16-I03",
        "EE-16",
        "IfStatement : if (Expression) Statement / Statement IsLabelledFunction",
        &["IsLabelledFunction"],
    ),
    // EE-17
    inactive_rule(
        "EE-17-I01",
        "EE-17",
        "DoWhileStatement / Statement IsLabelledFunction",
        &["IsLabelledFunction"],
    ),
    // EE-18
    inactive_rule(
        "EE-18-I01",
        "EE-18",
        "WhileStatement / Statement IsLabelledFunction",
        &["IsLabelledFunction"],
    ),
    // EE-19
    inactive_rule(
        "EE-19-I01",
        "EE-19",
        "ForStatement families / Statement IsLabelledFunction",
        &["IsLabelledFunction"],
    ),
    active_rule(
        "EE-19-R01",
        "EE-19",
        "ForStatement with LexicalDeclaration / BoundNames intersects VarDeclaredNames(Statement)",
        &["BoundNames", "VarDeclaredNames"],
        &[],
    ),
    // EE-20
    inactive_rule(
        "EE-20-I01",
        "EE-20",
        "ForInOfStatement families / Statement IsLabelledFunction",
        &["IsLabelledFunction"],
    ),
    active_rule(
        "EE-20-R01",
        "EE-20",
        "for-in/of LHS ObjectLiteral|ArrayLiteral / must cover AssignmentPattern",
        &["cover grammar reinterpretation"],
        &[],
    ),
    active_rule(
        "EE-20-R02",
        "EE-20",
        "for-in/of non-object-array LHS / AssignmentTargetType must not be invalid",
        &["AssignmentTargetType"],
        &[],
    ),
    active_rule(
        "EE-20-R03",
        "EE-20",
        "ForDeclaration / BoundNames must not contain let",
        &["BoundNames"],
        &[],
    ),
    active_rule(
        "EE-20-R04",
        "EE-20",
        "ForDeclaration / BoundNames intersects VarDeclaredNames(Statement)",
        &["BoundNames", "VarDeclaredNames"],
        &[],
    ),
    active_rule(
        "EE-20-R05",
        "EE-20",
        "ForDeclaration / BoundNames must not contain duplicates",
        &["BoundNames"],
        &[],
    ),
    // EE-21
    active_rule(
        "EE-21-R01",
        "EE-21",
        "ContinueStatement / must be nested in IterationStatement without crossing function or static-block boundary",
        &["syntactic containment"],
        &[],
    ),
    // EE-22
    active_rule(
        "EE-22-R01",
        "EE-22",
        "BreakStatement : break ; / must be nested in IterationStatement or SwitchStatement without crossing function/static-block boundary",
        &["syntactic containment"],
        &[],
    ),
    // EE-23
    active_rule(
        "EE-23-R01",
        "EE-23",
        "WithStatement / strict-mode rejection",
        &["IsStrict"],
        &["JS-GOLD-STRICT-WITH-001"],
    ),
    inactive_rule(
        "EE-23-I01",
        "EE-23",
        "WithStatement / Statement IsLabelledFunction",
        &["IsLabelledFunction"],
    ),
    // EE-24
    active_rule(
        "EE-24-R01",
        "EE-24",
        "CaseBlock / duplicate LexicallyDeclaredNames",
        &["LexicallyDeclaredNames"],
        &[],
    ),
    active_rule(
        "EE-24-R02",
        "EE-24",
        "CaseBlock / LexicallyDeclaredNames intersects VarDeclaredNames",
        &["LexicallyDeclaredNames", "VarDeclaredNames"],
        &[],
    ),
    // EE-25
    active_rule(
        "EE-25-R01",
        "EE-25",
        "LabelledItem : FunctionDeclaration / base rejection with web-legacy exception disabled",
        &["selected Normative Optional policy"],
        &[],
    ),
    // EE-26
    active_rule(
        "EE-26-R01",
        "EE-26",
        "Catch : catch (CatchParameter) Block / duplicate BoundNames(CatchParameter)",
        &["BoundNames"],
        &[],
    ),
    active_rule(
        "EE-26-R02",
        "EE-26",
        "Catch / BoundNames(CatchParameter) intersects LexicallyDeclaredNames(Block)",
        &["BoundNames", "LexicallyDeclaredNames"],
        &[],
    ),
    active_rule(
        "EE-26-R03",
        "EE-26",
        "Catch / BoundNames(CatchParameter) intersects VarDeclaredNames(Block), web-legacy exception disabled",
        &[
            "BoundNames",
            "VarDeclaredNames",
            "selected Normative Optional policy",
        ],
        &[],
    ),
    // EE-27
    active_rule(
        "EE-27-R01",
        "EE-27",
        "UniqueFormalParameters : FormalParameters / duplicate BoundNames",
        &["BoundNames"],
        &[],
    ),
    active_rule(
        "EE-27-R02",
        "EE-27",
        "FormalParameters / non-simple parameter list with duplicate BoundNames",
        &["IsSimpleParameterList", "BoundNames"],
        &[],
    ),
    // EE-28
    active_rule(
        "EE-28-R01",
        "EE-28",
        "FunctionDeclaration|FunctionExpression / strict FormalParameters apply UniqueFormalParameters Early Errors",
        &["IsStrict", "UniqueFormalParameters"],
        &[],
    ),
    active_rule(
        "EE-28-R02",
        "EE-28",
        "FunctionDeclaration|FunctionExpression / strict BindingIdentifier eval-or-arguments rejection",
        &["IsStrict", "StringValue"],
        &[],
    ),
    active_rule(
        "EE-28-R03",
        "EE-28",
        "FunctionDeclaration|FunctionExpression / Use Strict with non-simple FormalParameters",
        &["FunctionBodyContainsUseStrict", "IsSimpleParameterList"],
        &[],
    ),
    active_rule(
        "EE-28-R04",
        "EE-28",
        "FunctionDeclaration|FunctionExpression / BoundNames(FormalParameters) intersects LexicallyDeclaredNames(FunctionBody)",
        &["BoundNames", "LexicallyDeclaredNames"],
        &[],
    ),
    active_rule(
        "EE-28-R05",
        "EE-28",
        "FunctionDeclaration|FunctionExpression / FormalParameters Contains SuperProperty",
        &["Contains"],
        &[],
    ),
    active_rule(
        "EE-28-R06",
        "EE-28",
        "FunctionDeclaration|FunctionExpression / FunctionBody Contains SuperProperty",
        &["Contains"],
        &[],
    ),
    active_rule(
        "EE-28-R07",
        "EE-28",
        "FunctionDeclaration|FunctionExpression / FormalParameters Contains SuperCall",
        &["Contains"],
        &[],
    ),
    active_rule(
        "EE-28-R08",
        "EE-28",
        "FunctionDeclaration|FunctionExpression / FunctionBody Contains SuperCall",
        &["Contains"],
        &[],
    ),
    active_rule(
        "EE-28-R09",
        "EE-28",
        "FunctionBody / duplicate LexicallyDeclaredNames",
        &["LexicallyDeclaredNames"],
        &[],
    ),
    active_rule(
        "EE-28-R10",
        "EE-28",
        "FunctionBody / LexicallyDeclaredNames intersects VarDeclaredNames",
        &["LexicallyDeclaredNames", "VarDeclaredNames"],
        &[],
    ),
    active_rule(
        "EE-28-R11",
        "EE-28",
        "FunctionBody / ContainsDuplicateLabels",
        &["ContainsDuplicateLabels"],
        &[],
    ),
    active_rule(
        "EE-28-R12",
        "EE-28",
        "FunctionBody / ContainsUndefinedBreakTarget",
        &["ContainsUndefinedBreakTarget"],
        &[],
    ),
    active_rule(
        "EE-28-R13",
        "EE-28",
        "FunctionBody / ContainsUndefinedContinueTarget",
        &["ContainsUndefinedContinueTarget"],
        &[],
    ),
    // EE-29
    active_rule(
        "EE-29-R01",
        "EE-29",
        "ArrowFunction / ArrowParameters Contains YieldExpression",
        &["Contains"],
        &[],
    ),
    active_rule(
        "EE-29-R02",
        "EE-29",
        "ArrowFunction / ArrowParameters Contains AwaitExpression",
        &["Contains"],
        &[],
    ),
    active_rule(
        "EE-29-R03",
        "EE-29",
        "ArrowFunction / Use Strict with non-simple ArrowParameters",
        &["ConciseBodyContainsUseStrict", "IsSimpleParameterList"],
        &[],
    ),
    active_rule(
        "EE-29-R04",
        "EE-29",
        "ArrowFunction / BoundNames(ArrowParameters) intersects LexicallyDeclaredNames(ConciseBody)",
        &["BoundNames", "LexicallyDeclaredNames"],
        &[],
    ),
    active_rule(
        "EE-29-R05",
        "EE-29",
        "ArrowParameters : CoverParenthesizedExpressionAndArrowParameterList / must cover ArrowFormalParameters",
        &["cover grammar reinterpretation"],
        &[],
    ),
    // EE-30
    active_rule(
        "EE-30-R01",
        "EE-30",
        "MethodDefinition / Use Strict with non-simple UniqueFormalParameters",
        &["FunctionBodyContainsUseStrict", "IsSimpleParameterList"],
        &[],
    ),
    active_rule(
        "EE-30-R02",
        "EE-30",
        "MethodDefinition / BoundNames(UniqueFormalParameters) intersects LexicallyDeclaredNames(FunctionBody)",
        &["BoundNames", "LexicallyDeclaredNames"],
        &[],
    ),
    active_rule(
        "EE-30-R03",
        "EE-30",
        "SetterParameterList / duplicate BoundNames",
        &["BoundNames"],
        &[],
    ),
    active_rule(
        "EE-30-R04",
        "EE-30",
        "Setter MethodDefinition / Use Strict with non-simple parameter list",
        &["FunctionBodyContainsUseStrict", "IsSimpleParameterList"],
        &[],
    ),
    active_rule(
        "EE-30-R05",
        "EE-30",
        "Setter MethodDefinition / BoundNames(parameter) intersects LexicallyDeclaredNames(FunctionBody)",
        &["BoundNames", "LexicallyDeclaredNames"],
        &[],
    ),
    // EE-31
    active_rule(
        "EE-31-R01",
        "EE-31",
        "GeneratorMethod / HasDirectSuper",
        &["HasDirectSuper"],
        &[],
    ),
    active_rule(
        "EE-31-R02",
        "EE-31",
        "GeneratorMethod / UniqueFormalParameters Contains YieldExpression",
        &["Contains"],
        &[],
    ),
    active_rule(
        "EE-31-R03",
        "EE-31",
        "GeneratorMethod / Use Strict with non-simple parameters",
        &["FunctionBodyContainsUseStrict", "IsSimpleParameterList"],
        &[],
    ),
    active_rule(
        "EE-31-R04",
        "EE-31",
        "GeneratorMethod / BoundNames(parameters) intersects LexicallyDeclaredNames(GeneratorBody)",
        &["BoundNames", "LexicallyDeclaredNames"],
        &[],
    ),
    active_rule(
        "EE-31-R05",
        "EE-31",
        "GeneratorDeclaration|GeneratorExpression / strict FormalParameters apply UniqueFormalParameters Early Errors",
        &["IsStrict", "UniqueFormalParameters"],
        &[],
    ),
    active_rule(
        "EE-31-R06",
        "EE-31",
        "GeneratorDeclaration|GeneratorExpression / strict BindingIdentifier eval-or-arguments rejection",
        &["IsStrict", "StringValue"],
        &[],
    ),
    active_rule(
        "EE-31-R07",
        "EE-31",
        "GeneratorDeclaration|GeneratorExpression / Use Strict with non-simple FormalParameters",
        &["FunctionBodyContainsUseStrict", "IsSimpleParameterList"],
        &[],
    ),
    active_rule(
        "EE-31-R08",
        "EE-31",
        "GeneratorDeclaration|GeneratorExpression / BoundNames(FormalParameters) intersects LexicallyDeclaredNames(GeneratorBody)",
        &["BoundNames", "LexicallyDeclaredNames"],
        &[],
    ),
    active_rule(
        "EE-31-R09",
        "EE-31",
        "GeneratorDeclaration|GeneratorExpression / FormalParameters Contains YieldExpression",
        &["Contains"],
        &[],
    ),
    active_rule(
        "EE-31-R10",
        "EE-31",
        "GeneratorDeclaration|GeneratorExpression / FormalParameters Contains SuperProperty",
        &["Contains"],
        &[],
    ),
    active_rule(
        "EE-31-R11",
        "EE-31",
        "GeneratorDeclaration|GeneratorExpression / GeneratorBody Contains SuperProperty",
        &["Contains"],
        &[],
    ),
    active_rule(
        "EE-31-R12",
        "EE-31",
        "GeneratorDeclaration|GeneratorExpression / FormalParameters Contains SuperCall",
        &["Contains"],
        &[],
    ),
    active_rule(
        "EE-31-R13",
        "EE-31",
        "GeneratorDeclaration|GeneratorExpression / GeneratorBody Contains SuperCall",
        &["Contains"],
        &[],
    ),
    // EE-32
    active_rule(
        "EE-32-R01",
        "EE-32",
        "AsyncGeneratorMethod / HasDirectSuper",
        &["HasDirectSuper"],
        &[],
    ),
    active_rule(
        "EE-32-R02",
        "EE-32",
        "AsyncGeneratorMethod / UniqueFormalParameters Contains YieldExpression",
        &["Contains"],
        &[],
    ),
    active_rule(
        "EE-32-R03",
        "EE-32",
        "AsyncGeneratorMethod / UniqueFormalParameters Contains AwaitExpression",
        &["Contains"],
        &[],
    ),
    active_rule(
        "EE-32-R04",
        "EE-32",
        "AsyncGeneratorMethod / Use Strict with non-simple parameters",
        &["FunctionBodyContainsUseStrict", "IsSimpleParameterList"],
        &[],
    ),
    active_rule(
        "EE-32-R05",
        "EE-32",
        "AsyncGeneratorMethod / BoundNames(parameters) intersects LexicallyDeclaredNames(AsyncGeneratorBody)",
        &["BoundNames", "LexicallyDeclaredNames"],
        &[],
    ),
    active_rule(
        "EE-32-R06",
        "EE-32",
        "AsyncGeneratorDeclaration|Expression / strict FormalParameters apply UniqueFormalParameters Early Errors",
        &["IsStrict", "UniqueFormalParameters"],
        &[],
    ),
    active_rule(
        "EE-32-R07",
        "EE-32",
        "AsyncGeneratorDeclaration|Expression / strict BindingIdentifier eval-or-arguments rejection",
        &["IsStrict", "StringValue"],
        &[],
    ),
    active_rule(
        "EE-32-R08",
        "EE-32",
        "AsyncGeneratorDeclaration|Expression / Use Strict with non-simple FormalParameters",
        &["FunctionBodyContainsUseStrict", "IsSimpleParameterList"],
        &[],
    ),
    active_rule(
        "EE-32-R09",
        "EE-32",
        "AsyncGeneratorDeclaration|Expression / BoundNames(FormalParameters) intersects LexicallyDeclaredNames(AsyncGeneratorBody)",
        &["BoundNames", "LexicallyDeclaredNames"],
        &[],
    ),
    active_rule(
        "EE-32-R10",
        "EE-32",
        "AsyncGeneratorDeclaration|Expression / FormalParameters Contains YieldExpression",
        &["Contains"],
        &[],
    ),
    active_rule(
        "EE-32-R11",
        "EE-32",
        "AsyncGeneratorDeclaration|Expression / FormalParameters Contains AwaitExpression",
        &["Contains"],
        &[],
    ),
    active_rule(
        "EE-32-R12",
        "EE-32",
        "AsyncGeneratorDeclaration|Expression / FormalParameters Contains SuperProperty",
        &["Contains"],
        &[],
    ),
    active_rule(
        "EE-32-R13",
        "EE-32",
        "AsyncGeneratorDeclaration|Expression / AsyncGeneratorBody Contains SuperProperty",
        &["Contains"],
        &[],
    ),
    active_rule(
        "EE-32-R14",
        "EE-32",
        "AsyncGeneratorDeclaration|Expression / FormalParameters Contains SuperCall",
        &["Contains"],
        &[],
    ),
    active_rule(
        "EE-32-R15",
        "EE-32",
        "AsyncGeneratorDeclaration|Expression / AsyncGeneratorBody Contains SuperCall",
        &["Contains"],
        &[],
    ),
    // EE-33
    active_rule(
        "EE-33-R01",
        "EE-33",
        "ClassTail without heritage / constructor HasDirectSuper",
        &["ConstructorMethod", "HasDirectSuper"],
        &[],
    ),
    active_rule(
        "EE-33-R02",
        "EE-33",
        "ClassBody / PrototypePropertyNameList contains constructor more than once",
        &["PrototypePropertyNameList"],
        &[],
    ),
    active_rule(
        "EE-33-R03",
        "EE-33",
        "ClassBody / duplicate PrivateBoundIdentifiers except valid getter-setter pair",
        &["PrivateBoundIdentifiers"],
        &[],
    ),
    active_rule(
        "EE-33-R04",
        "EE-33",
        "ClassElement : MethodDefinition / non-constructor HasDirectSuper",
        &["PropName", "HasDirectSuper"],
        &[],
    ),
    active_rule(
        "EE-33-R05",
        "EE-33",
        "ClassElement : MethodDefinition / constructor SpecialMethod",
        &["PropName", "SpecialMethod"],
        &[],
    ),
    active_rule(
        "EE-33-R06",
        "EE-33",
        "ClassElement : static MethodDefinition / HasDirectSuper",
        &["HasDirectSuper"],
        &[],
    ),
    active_rule(
        "EE-33-R07",
        "EE-33",
        "ClassElement : static MethodDefinition / PropName prototype",
        &["PropName"],
        &[],
    ),
    active_rule(
        "EE-33-R08",
        "EE-33",
        "ClassElement : FieldDefinition / PropName constructor",
        &["PropName"],
        &[],
    ),
    active_rule(
        "EE-33-R09",
        "EE-33",
        "ClassElement : static FieldDefinition / PropName prototype-or-constructor",
        &["PropName"],
        &[],
    ),
    active_rule(
        "EE-33-R10",
        "EE-33",
        "FieldDefinition / Initializer ContainsArguments",
        &["ContainsArguments"],
        &[],
    ),
    active_rule(
        "EE-33-R11",
        "EE-33",
        "FieldDefinition / Initializer Contains SuperCall",
        &["Contains"],
        &[],
    ),
    active_rule(
        "EE-33-R12",
        "EE-33",
        "ClassElementName : PrivateIdentifier / StringValue #constructor",
        &["StringValue"],
        &[],
    ),
    active_rule(
        "EE-33-R13",
        "EE-33",
        "ClassStaticBlockBody / duplicate LexicallyDeclaredNames",
        &["LexicallyDeclaredNames"],
        &[],
    ),
    active_rule(
        "EE-33-R14",
        "EE-33",
        "ClassStaticBlockBody / LexicallyDeclaredNames intersects VarDeclaredNames",
        &["LexicallyDeclaredNames", "VarDeclaredNames"],
        &[],
    ),
    active_rule(
        "EE-33-R15",
        "EE-33",
        "ClassStaticBlockBody / ContainsDuplicateLabels",
        &["ContainsDuplicateLabels"],
        &[],
    ),
    active_rule(
        "EE-33-R16",
        "EE-33",
        "ClassStaticBlockBody / ContainsUndefinedBreakTarget",
        &["ContainsUndefinedBreakTarget"],
        &[],
    ),
    active_rule(
        "EE-33-R17",
        "EE-33",
        "ClassStaticBlockBody / ContainsUndefinedContinueTarget",
        &["ContainsUndefinedContinueTarget"],
        &[],
    ),
    active_rule(
        "EE-33-R18",
        "EE-33",
        "ClassStaticBlockBody / ContainsArguments",
        &["ContainsArguments"],
        &[],
    ),
    active_rule(
        "EE-33-R19",
        "EE-33",
        "ClassStaticBlockBody / Contains SuperCall",
        &["Contains"],
        &[],
    ),
    active_rule(
        "EE-33-R20",
        "EE-33",
        "ClassStaticBlockBody / Contains await",
        &["Contains"],
        &[],
    ),
    // EE-34
    active_rule(
        "EE-34-R01",
        "EE-34",
        "AsyncMethod / Use Strict with non-simple parameters",
        &["FunctionBodyContainsUseStrict", "IsSimpleParameterList"],
        &[],
    ),
    active_rule(
        "EE-34-R02",
        "EE-34",
        "AsyncMethod / HasDirectSuper",
        &["HasDirectSuper"],
        &[],
    ),
    active_rule(
        "EE-34-R03",
        "EE-34",
        "AsyncMethod / UniqueFormalParameters Contains AwaitExpression",
        &["Contains"],
        &[],
    ),
    active_rule(
        "EE-34-R04",
        "EE-34",
        "AsyncMethod / BoundNames(parameters) intersects LexicallyDeclaredNames(AsyncFunctionBody)",
        &["BoundNames", "LexicallyDeclaredNames"],
        &[],
    ),
    active_rule(
        "EE-34-R05",
        "EE-34",
        "AsyncFunctionDeclaration|Expression / Use Strict with non-simple FormalParameters",
        &["FunctionBodyContainsUseStrict", "IsSimpleParameterList"],
        &[],
    ),
    active_rule(
        "EE-34-R06",
        "EE-34",
        "AsyncFunctionDeclaration|Expression / FormalParameters Contains AwaitExpression",
        &["Contains"],
        &[],
    ),
    active_rule(
        "EE-34-R07",
        "EE-34",
        "AsyncFunctionDeclaration|Expression / strict FormalParameters apply UniqueFormalParameters Early Errors",
        &["IsStrict", "UniqueFormalParameters"],
        &[],
    ),
    active_rule(
        "EE-34-R08",
        "EE-34",
        "AsyncFunctionDeclaration|Expression / strict BindingIdentifier eval-or-arguments rejection",
        &["IsStrict", "StringValue"],
        &[],
    ),
    active_rule(
        "EE-34-R09",
        "EE-34",
        "AsyncFunctionDeclaration|Expression / BoundNames(FormalParameters) intersects LexicallyDeclaredNames(AsyncFunctionBody)",
        &["BoundNames", "LexicallyDeclaredNames"],
        &[],
    ),
    active_rule(
        "EE-34-R10",
        "EE-34",
        "AsyncFunctionDeclaration|Expression / FormalParameters Contains SuperProperty",
        &["Contains"],
        &[],
    ),
    active_rule(
        "EE-34-R11",
        "EE-34",
        "AsyncFunctionDeclaration|Expression / AsyncFunctionBody Contains SuperProperty",
        &["Contains"],
        &[],
    ),
    active_rule(
        "EE-34-R12",
        "EE-34",
        "AsyncFunctionDeclaration|Expression / FormalParameters Contains SuperCall",
        &["Contains"],
        &[],
    ),
    active_rule(
        "EE-34-R13",
        "EE-34",
        "AsyncFunctionDeclaration|Expression / AsyncFunctionBody Contains SuperCall",
        &["Contains"],
        &[],
    ),
    // EE-35
    active_rule(
        "EE-35-R01",
        "EE-35",
        "AsyncArrowFunction : async AsyncArrowBindingIdentifier => AsyncConciseBody / BoundNames intersects LexicallyDeclaredNames",
        &["BoundNames", "LexicallyDeclaredNames"],
        &[],
    ),
    active_rule(
        "EE-35-R02",
        "EE-35",
        "AsyncArrowFunction : CoverCallExpressionAndAsyncArrowHead => AsyncConciseBody / cover must be AsyncArrowHead",
        &["cover grammar reinterpretation"],
        &[],
    ),
    active_rule(
        "EE-35-R03",
        "EE-35",
        "AsyncArrowFunction cover head / Contains YieldExpression",
        &["Contains"],
        &[],
    ),
    active_rule(
        "EE-35-R04",
        "EE-35",
        "AsyncArrowFunction cover head / Contains AwaitExpression",
        &["Contains"],
        &[],
    ),
    active_rule(
        "EE-35-R05",
        "EE-35",
        "AsyncArrowFunction cover head / BoundNames intersects LexicallyDeclaredNames",
        &["BoundNames", "LexicallyDeclaredNames"],
        &[],
    ),
    active_rule(
        "EE-35-R06",
        "EE-35",
        "AsyncArrowFunction cover head / Use Strict with non-simple parameter list",
        &["AsyncConciseBodyContainsUseStrict", "IsSimpleParameterList"],
        &[],
    ),
    // EE-36
    active_rule(
        "EE-36-R01",
        "EE-36",
        "Script : ScriptBody / duplicate LexicallyDeclaredNames",
        &["LexicallyDeclaredNames"],
        &["JS-GOLD-SCRIPT-DUPLEXICAL-001"],
    ),
    active_rule(
        "EE-36-R02",
        "EE-36",
        "Script : ScriptBody / LexicallyDeclaredNames intersects VarDeclaredNames",
        &["LexicallyDeclaredNames", "VarDeclaredNames"],
        &[],
    ),
    active_rule(
        "EE-36-R03",
        "EE-36",
        "ScriptBody : StatementList / Contains super outside direct eval",
        &["Contains", "Independent Source Unit"],
        &[],
    ),
    active_rule(
        "EE-36-R04",
        "EE-36",
        "ScriptBody : StatementList / Contains NewTarget outside direct eval",
        &["Contains", "Independent Source Unit"],
        &[],
    ),
    active_rule(
        "EE-36-R05",
        "EE-36",
        "ScriptBody : StatementList / ContainsDuplicateLabels",
        &["ContainsDuplicateLabels"],
        &[],
    ),
    active_rule(
        "EE-36-R06",
        "EE-36",
        "ScriptBody : StatementList / ContainsUndefinedBreakTarget",
        &["ContainsUndefinedBreakTarget"],
        &[],
    ),
    active_rule(
        "EE-36-R07",
        "EE-36",
        "ScriptBody : StatementList / ContainsUndefinedContinueTarget",
        &["ContainsUndefinedContinueTarget"],
        &[],
    ),
    active_rule(
        "EE-36-R08",
        "EE-36",
        "ScriptBody : StatementList / AllPrivateIdentifiersValid outside direct eval",
        &["AllPrivateIdentifiersValid", "Independent Source Unit"],
        &[],
    ),
    // EE-37
    active_rule(
        "EE-37-R01",
        "EE-37",
        "Pattern :: Disjunction / capturing-group count must be < 2^32 - 1",
        &["CountLeftCapturingParensWithin"],
        &[],
    ),
    active_rule(
        "EE-37-R02",
        "EE-37",
        "Pattern :: Disjunction / duplicate named groups that MightBothParticipate",
        &["CapturingGroupName", "MightBothParticipate"],
        &[],
    ),
    active_rule(
        "EE-37-R03",
        "EE-37",
        "QuantifierPrefix {m,n} / first MV must not exceed second MV",
        &["MV"],
        &[],
    ),
    active_rule(
        "EE-37-R04",
        "EE-37",
        "Atom modifier group / RegularExpressionModifiers contains duplicate code point",
        &[],
        &[],
    ),
    active_rule(
        "EE-37-R05",
        "EE-37",
        "Atom modifier-minus group / both modifier sets must not be empty",
        &[],
        &[],
    ),
    active_rule(
        "EE-37-R06",
        "EE-37",
        "Atom modifier-minus group / first modifiers contains duplicate code point",
        &[],
        &[],
    ),
    active_rule(
        "EE-37-R07",
        "EE-37",
        "Atom modifier-minus group / second modifiers contains duplicate code point",
        &[],
        &[],
    ),
    active_rule(
        "EE-37-R08",
        "EE-37",
        "Atom modifier-minus group / modifier sets must not overlap",
        &[],
        &[],
    ),
    active_rule(
        "EE-37-R09",
        "EE-37",
        "AtomEscape : k GroupName / GroupSpecifiersThatMatch must be non-empty",
        &["GroupSpecifiersThatMatch"],
        &[],
    ),
    active_rule(
        "EE-37-R10",
        "EE-37",
        "AtomEscape : DecimalEscape / CapturingGroupNumber must not exceed enclosing count",
        &["CapturingGroupNumber", "CountLeftCapturingParensWithin"],
        &[],
    ),
    active_rule(
        "EE-37-R11",
        "EE-37",
        "NonemptyClassRanges / range endpoints must not be character classes",
        &["IsCharacterClass"],
        &[],
    ),
    active_rule(
        "EE-37-R12",
        "EE-37",
        "NonemptyClassRanges / lower CharacterValue must not exceed upper",
        &["IsCharacterClass", "CharacterValue"],
        &[],
    ),
    active_rule(
        "EE-37-R13",
        "EE-37",
        "NonemptyClassRangesNoDash / range endpoints must not be character classes",
        &["IsCharacterClass"],
        &[],
    ),
    active_rule(
        "EE-37-R14",
        "EE-37",
        "NonemptyClassRangesNoDash / lower CharacterValue must not exceed upper",
        &["IsCharacterClass", "CharacterValue"],
        &[],
    ),
    active_rule(
        "EE-37-R15",
        "EE-37",
        "RegExpIdentifierStart escape / CharacterValue must match IdentifierStartChar",
        &["CharacterValue", "Unicode ID_Start"],
        &[],
    ),
    active_rule(
        "EE-37-R16",
        "EE-37",
        "RegExpIdentifierStart surrogate pair / code point must match UnicodeIDStart",
        &["RegExpIdentifierCodePoint", "Unicode ID_Start"],
        &[],
    ),
    active_rule(
        "EE-37-R17",
        "EE-37",
        "RegExpIdentifierPart escape / CharacterValue must match IdentifierPartChar",
        &["CharacterValue", "Unicode ID_Continue"],
        &[],
    ),
    active_rule(
        "EE-37-R18",
        "EE-37",
        "RegExpIdentifierPart surrogate pair / code point must match UnicodeIDContinue",
        &["RegExpIdentifierCodePoint", "Unicode ID_Continue"],
        &[],
    ),
    active_rule(
        "EE-37-R19",
        "EE-37",
        "UnicodePropertyName=UnicodePropertyValue / property name or alias must be allowed",
        &["Unicode property-name vocabulary"],
        &[],
    ),
    active_rule(
        "EE-37-R20",
        "EE-37",
        "UnicodePropertyName=UnicodePropertyValue / property value or alias must match selected property",
        &["Unicode property-value vocabulary"],
        &[],
    ),
    active_rule(
        "EE-37-R21",
        "EE-37",
        "LoneUnicodePropertyNameOrValue / must be gc value, binary property, or binary property of strings",
        &[
            "Unicode property-value vocabulary",
            "binary property vocabulary",
        ],
        &["JS-GOLD-REGEXP-BAD-PROPERTY-001"],
    ),
    active_rule(
        "EE-37-R22",
        "EE-37",
        "LoneUnicodePropertyNameOrValue / binary property of strings requires UnicodeSetsMode",
        &[
            "UnicodeSetsMode",
            "binary property-of-strings classification",
        ],
        &[],
    ),
    active_rule(
        "EE-37-R23",
        "EE-37",
        "CharacterClassEscape : P{...} / MayContainStrings must be false",
        &["MayContainStrings"],
        &[],
    ),
    active_rule(
        "EE-37-R24",
        "EE-37",
        "CharacterClass : [^ ClassContents] / MayContainStrings must be false",
        &["MayContainStrings"],
        &[],
    ),
    active_rule(
        "EE-37-R25",
        "EE-37",
        "NestedClass : [^ ClassContents] / MayContainStrings must be false",
        &["MayContainStrings"],
        &[],
    ),
    active_rule(
        "EE-37-R26",
        "EE-37",
        "ClassSetRange / first CharacterValue must not exceed second",
        &["CharacterValue"],
        &[],
    ),
];

const fn active_rule(
    id: &'static str,
    container_id: &'static str,
    normative_locator: &'static str,
    dependencies: &'static [&'static str],
    gold_fixture_ids: &'static [&'static str],
) -> RuleUnit {
    RuleUnit {
        id,
        container_id,
        kind: RuleUnitKind::NormativeRule,
        normative_locator,
        dependencies,
        gold_obligations: ACTIVE_GOLD,
        gold_fixture_ids,
    }
}

const fn inactive_rule(
    id: &'static str,
    container_id: &'static str,
    normative_locator: &'static str,
    dependencies: &'static [&'static str],
) -> RuleUnit {
    RuleUnit {
        id,
        container_id,
        kind: RuleUnitKind::EnvelopeInactiveRule,
        normative_locator,
        dependencies,
        gold_obligations: INACTIVE_GOLD,
        gold_fixture_ids: &[],
    }
}

#[allow(dead_code)]
const fn sentinel(id: &'static str, container_id: &'static str) -> RuleUnit {
    RuleUnit {
        id,
        container_id,
        kind: RuleUnitKind::ExpansionSentinel,
        normative_locator: "pinned-snapshot rule expansion pending",
        dependencies: &[],
        gold_obligations: EXPANSION_REQUIRED,
        gold_fixture_ids: &[],
    }
}

pub(crate) fn validate_inventory(
    containers: &[EarlyErrorContainer],
    rule_units: &[RuleUnit],
) -> Result<(), InventoryError> {
    let expected: BTreeSet<&str> = CONTAINERS.iter().map(|container| container.id).collect();
    let mut seen_containers = BTreeSet::new();

    for container in containers {
        if !seen_containers.insert(container.id) {
            return Err(InventoryError::DuplicateContainer(container.id));
        }
    }

    for expected_id in &expected {
        if !seen_containers.contains(expected_id) {
            return Err(InventoryError::MissingContainer(expected_id));
        }
    }

    let mut seen_rules = BTreeSet::new();
    let mut containers_with_rules = BTreeSet::new();
    for rule_unit in rule_units {
        if !seen_containers.contains(rule_unit.container_id) {
            return Err(InventoryError::UnknownContainerForRule(rule_unit.id));
        }
        containers_with_rules.insert(rule_unit.container_id);
        if !seen_rules.insert(rule_unit.id) {
            return Err(InventoryError::DuplicateRuleUnit(rule_unit.id));
        }
        if rule_unit.normative_locator.is_empty() {
            return Err(InventoryError::EmptyNormativeLocator(rule_unit.id));
        }
        if rule_unit.gold_obligations.is_empty() {
            return Err(InventoryError::MissingGoldObligations(rule_unit.id));
        }
    }

    for expected_id in expected {
        if !containers_with_rules.contains(expected_id) {
            return Err(InventoryError::MissingRuleUnitForContainer(expected_id));
        }
    }

    Ok(())
}

pub(crate) fn rule_units_by_container(rule_units: &[RuleUnit]) -> BTreeMap<&str, Vec<&RuleUnit>> {
    let mut result: BTreeMap<&str, Vec<&RuleUnit>> = BTreeMap::new();
    for rule_unit in rule_units {
        result
            .entry(rule_unit.container_id)
            .or_default()
            .push(rule_unit);
    }
    result
}

pub(crate) fn aggregate_rule_expansion_complete(rule_units: &[RuleUnit]) -> bool {
    !rule_units
        .iter()
        .any(|rule_unit| rule_unit.kind == RuleUnitKind::ExpansionSentinel)
}

pub(crate) fn aggregate_gold_coverage_complete(rule_units: &[RuleUnit]) -> bool {
    rule_units.iter().all(|rule_unit| {
        rule_unit.kind != RuleUnitKind::NormativeRule || !rule_unit.gold_fixture_ids.is_empty()
    })
}

pub(crate) fn active_rule_container_count(rule_units: &[RuleUnit]) -> usize {
    rule_units
        .iter()
        .filter(|rule_unit| rule_unit.kind == RuleUnitKind::NormativeRule)
        .map(|rule_unit| rule_unit.container_id)
        .collect::<BTreeSet<_>>()
        .len()
}
