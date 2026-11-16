// SPDX-FileCopyrightText: 2026 Blackcat Informatics(R) Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! SEAM-SCOPED extract: verbatim const block only (upstream
//! crates/xsd/src/datatype.rs lines 13-136, rev 0da05783). The full datatype
//! enum/impl surface (exact/value/bigint closure) is W1 stardf-xsd scope and is
//! deliberately NOT approximated here. Pending replacement by full W1 stardf-xsd.

/// The XML Schema datatype namespace.
pub const XSD_NS: &str = "http://www.w3.org/2001/XMLSchema#";

/// `xsd:integer` — arbitrary-magnitude (this crate: `i128`-bounded) signed integer.
pub const XSD_INTEGER: &str = "http://www.w3.org/2001/XMLSchema#integer";
/// `xsd:long` — 64-bit signed integer (`-9223372036854775808..9223372036854775807`).
pub const XSD_LONG: &str = "http://www.w3.org/2001/XMLSchema#long";
/// `xsd:int` — 32-bit signed integer (`-2147483648..2147483647`).
pub const XSD_INT: &str = "http://www.w3.org/2001/XMLSchema#int";
/// `xsd:short` — 16-bit signed integer (`-32768..32767`).
pub const XSD_SHORT: &str = "http://www.w3.org/2001/XMLSchema#short";
/// `xsd:byte` — 8-bit signed integer (`-128..127`).
pub const XSD_BYTE: &str = "http://www.w3.org/2001/XMLSchema#byte";
/// `xsd:unsignedLong` — 64-bit unsigned integer (`0..18446744073709551615`).
pub const XSD_UNSIGNED_LONG: &str = "http://www.w3.org/2001/XMLSchema#unsignedLong";
/// `xsd:unsignedInt` — 32-bit unsigned integer (`0..4294967295`).
pub const XSD_UNSIGNED_INT: &str = "http://www.w3.org/2001/XMLSchema#unsignedInt";
/// `xsd:unsignedShort` — 16-bit unsigned integer (`0..65535`).
pub const XSD_UNSIGNED_SHORT: &str = "http://www.w3.org/2001/XMLSchema#unsignedShort";
/// `xsd:unsignedByte` — 8-bit unsigned integer (`0..255`).
pub const XSD_UNSIGNED_BYTE: &str = "http://www.w3.org/2001/XMLSchema#unsignedByte";
/// `xsd:nonNegativeInteger` — integer `>= 0`.
pub const XSD_NON_NEGATIVE_INTEGER: &str = "http://www.w3.org/2001/XMLSchema#nonNegativeInteger";
/// `xsd:positiveInteger` — integer `> 0` (i.e. `>= 1`).
pub const XSD_POSITIVE_INTEGER: &str = "http://www.w3.org/2001/XMLSchema#positiveInteger";
/// `xsd:nonPositiveInteger` — integer `<= 0`.
pub const XSD_NON_POSITIVE_INTEGER: &str = "http://www.w3.org/2001/XMLSchema#nonPositiveInteger";
/// `xsd:negativeInteger` — integer `< 0` (i.e. `<= -1`).
pub const XSD_NEGATIVE_INTEGER: &str = "http://www.w3.org/2001/XMLSchema#negativeInteger";
/// `xsd:decimal` — exact decimal (this crate: `i128` mantissa, fixed scale).
pub const XSD_DECIMAL: &str = "http://www.w3.org/2001/XMLSchema#decimal";
/// `xsd:float` — IEEE single-precision.
pub const XSD_FLOAT: &str = "http://www.w3.org/2001/XMLSchema#float";
/// `xsd:double` — IEEE double-precision.
pub const XSD_DOUBLE: &str = "http://www.w3.org/2001/XMLSchema#double";
/// `xsd:boolean`.
pub const XSD_BOOLEAN: &str = "http://www.w3.org/2001/XMLSchema#boolean";
/// `xsd:string`.
pub const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";
/// `xsd:date`.
pub const XSD_DATE: &str = "http://www.w3.org/2001/XMLSchema#date";
/// `xsd:time`.
pub const XSD_TIME: &str = "http://www.w3.org/2001/XMLSchema#time";
/// `xsd:dateTime`.
pub const XSD_DATE_TIME: &str = "http://www.w3.org/2001/XMLSchema#dateTime";
/// `xsd:duration` — the general duration (months + seconds; partial order).
pub const XSD_DURATION: &str = "http://www.w3.org/2001/XMLSchema#duration";
/// `xsd:dayTimeDuration` — totally-ordered duration subtype (seconds only).
pub const XSD_DAY_TIME_DURATION: &str = "http://www.w3.org/2001/XMLSchema#dayTimeDuration";
/// `xsd:yearMonthDuration` — totally-ordered duration subtype (months only).
pub const XSD_YEAR_MONTH_DURATION: &str = "http://www.w3.org/2001/XMLSchema#yearMonthDuration";
/// `xsd:gYear` — a Gregorian year (e.g. `2024`).
pub const XSD_G_YEAR: &str = "http://www.w3.org/2001/XMLSchema#gYear";
/// `xsd:gMonth` — a Gregorian month (e.g. `--05`).
pub const XSD_G_MONTH: &str = "http://www.w3.org/2001/XMLSchema#gMonth";
/// `xsd:gDay` — a Gregorian day of the month (e.g. `---15`).
pub const XSD_G_DAY: &str = "http://www.w3.org/2001/XMLSchema#gDay";
/// `xsd:gYearMonth` — a Gregorian year + month (e.g. `2024-05`).
pub const XSD_G_YEAR_MONTH: &str = "http://www.w3.org/2001/XMLSchema#gYearMonth";
/// `xsd:gMonthDay` — a Gregorian month + day (e.g. `--02-29`).
pub const XSD_G_MONTH_DAY: &str = "http://www.w3.org/2001/XMLSchema#gMonthDay";
/// `xsd:hexBinary` — a sequence of hex-encoded bytes.
pub const XSD_HEX_BINARY: &str = "http://www.w3.org/2001/XMLSchema#hexBinary";
/// `xsd:base64Binary` — a Base64-encoded byte sequence.
pub const XSD_BASE64_BINARY: &str = "http://www.w3.org/2001/XMLSchema#base64Binary";

/// `xsd:anyURI` — an IRI reference.
pub const XSD_ANY_URI: &str = "http://www.w3.org/2001/XMLSchema#anyURI";
/// `xsd:normalizedString` — a string with no carriage return, line feed or tab.
pub const XSD_NORMALIZED_STRING: &str = "http://www.w3.org/2001/XMLSchema#normalizedString";
/// `xsd:token` — a normalized string with no leading, trailing or doubled space.
pub const XSD_TOKEN: &str = "http://www.w3.org/2001/XMLSchema#token";
/// `xsd:language` — a BCP 47 language tag.
pub const XSD_LANGUAGE: &str = "http://www.w3.org/2001/XMLSchema#language";
/// `xsd:Name` — an XML `Name`.
pub const XSD_NAME: &str = "http://www.w3.org/2001/XMLSchema#Name";
/// `xsd:NCName` — an XML non-colonized name.
pub const XSD_NCNAME: &str = "http://www.w3.org/2001/XMLSchema#NCName";
/// `xsd:NMTOKEN` — an XML name token.
pub const XSD_NMTOKEN: &str = "http://www.w3.org/2001/XMLSchema#NMTOKEN";
/// `xsd:dateTimeStamp` — an `xsd:dateTime` whose timezone is required.
pub const XSD_DATE_TIME_STAMP: &str = "http://www.w3.org/2001/XMLSchema#dateTimeStamp";

// The constraining facets of XML Schema 1.1 Part 2 §4.3, which OWL 2 datatype
// restrictions and SHACL name as predicates.

/// `xsd:length` — the exact length facet.
pub const XSD_LENGTH: &str = "http://www.w3.org/2001/XMLSchema#length";
/// `xsd:minLength` — the minimum length facet.
pub const XSD_MIN_LENGTH: &str = "http://www.w3.org/2001/XMLSchema#minLength";
/// `xsd:maxLength` — the maximum length facet.
pub const XSD_MAX_LENGTH: &str = "http://www.w3.org/2001/XMLSchema#maxLength";
/// `xsd:pattern` — the regular-expression facet.
pub const XSD_PATTERN: &str = "http://www.w3.org/2001/XMLSchema#pattern";
/// `xsd:enumeration` — the enumerated-values facet.
pub const XSD_ENUMERATION: &str = "http://www.w3.org/2001/XMLSchema#enumeration";
/// `xsd:whiteSpace` — the whitespace-normalization facet.
pub const XSD_WHITE_SPACE: &str = "http://www.w3.org/2001/XMLSchema#whiteSpace";
/// `xsd:maxInclusive` — the inclusive upper bound facet.
pub const XSD_MAX_INCLUSIVE: &str = "http://www.w3.org/2001/XMLSchema#maxInclusive";
/// `xsd:maxExclusive` — the exclusive upper bound facet.
pub const XSD_MAX_EXCLUSIVE: &str = "http://www.w3.org/2001/XMLSchema#maxExclusive";
/// `xsd:minInclusive` — the inclusive lower bound facet.
pub const XSD_MIN_INCLUSIVE: &str = "http://www.w3.org/2001/XMLSchema#minInclusive";
/// `xsd:minExclusive` — the exclusive lower bound facet.
pub const XSD_MIN_EXCLUSIVE: &str = "http://www.w3.org/2001/XMLSchema#minExclusive";
/// `xsd:totalDigits` — the total-digits facet.
pub const XSD_TOTAL_DIGITS: &str = "http://www.w3.org/2001/XMLSchema#totalDigits";
/// `xsd:fractionDigits` — the fraction-digits facet.
pub const XSD_FRACTION_DIGITS: &str = "http://www.w3.org/2001/XMLSchema#fractionDigits";
/// `xsd:assertions` — the assertions facet.
pub const XSD_ASSERTIONS: &str = "http://www.w3.org/2001/XMLSchema#assertions";
/// `xsd:explicitTimezone` — the explicit-timezone facet.
pub const XSD_EXPLICIT_TIMEZONE: &str = "http://www.w3.org/2001/XMLSchema#explicitTimezone";

// The two numeric datatypes the OWL 2 datatype map adds to XSD's (OWL 2
// Structural Specification §4.1). They sit here, with the value space this
// crate models for them, rather than in `purrdf_iri::vocab::owl`.

/// `owl:real` — the real numbers (OWL 2 Structural Specification §4.1).
pub const OWL_REAL: &str = "http://www.w3.org/2002/07/owl#real";
/// `owl:rational` — the rational numbers (OWL 2 Structural Specification §4.1).
pub const OWL_RATIONAL: &str = "http://www.w3.org/2002/07/owl#rational";

