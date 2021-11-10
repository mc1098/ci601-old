use crate::http::{utils, StatusCode};

/// Header field name as defined in [RFC7230 Section
/// 3.2](https://datatracker.ietf.org/doc/html/rfc7230#section-3.2)
///
/// ```text
/// field-name = token
///
/// token = 1*pchar
/// ```
#[derive(Debug, Eq, Hash, PartialEq)]
pub enum HeaderFieldName {
    /// Represents static [`RegisteredFieldName`] values for known field names
    ///
    /// These are static as const instances can be found in the [`HeaderFieldName`] type.
    Registered(StaticFieldName),
    /// Represents unknown custom field names.
    Custom(String),
}

macro_rules! standard_field_name_impl {
    ($(
        $(#[$var_doc:meta])+
        $variant:ident, $static_ident:ident, $name:literal,
    )*) => {

        /// Represents known registered field names as per the [Hypertext Transfer Protocol (HTTP) Field
        /// Name Registry](https://www.iana.org/assignments/http-fields/http-fields.xhtml#field-names).
        #[derive(Debug, Eq, Hash, PartialEq)]
        #[non_exhaustive]
        pub enum StaticFieldName {
            $(
                $(#[$var_doc])+
                #[allow(non_camel_case_types)]
                $variant,
            )*
        }


        impl HeaderFieldName {

            $(
                $(#[$var_doc])+
                #[allow(non_camel_case_types)]
                pub const $static_ident: HeaderFieldName = HeaderFieldName::Registered(
                    StaticFieldName::$variant
                );
            )*

            /// Derive a [`HeaderFieldName`] from a slice of bytes.
            ///
            /// Returns a [`StatusCode::BAD_REQUEST`] when the slice of bytes does not match the ABNF
            /// syntax of [`HeaderFieldName`].
            pub fn from_bytes(src: &[u8]) -> Result<Self, StatusCode> {
                if src.is_empty() || !src.iter().copied().all(utils::abnf::is_tchar) {
                    return Err(StatusCode::BAD_REQUEST);
                }

                // SAFETY:
                // src slice contains all tchars which are valid ascii characters and
                // ascii characters are valid UTF-8 so this is satisfies the safety requirements
                // of from_utf8_unchecked.
                let token = unsafe { std::str::from_utf8_unchecked(src) }.to_ascii_lowercase();
                match token.as_ref() {
                    $($name => Ok(HeaderFieldName::$static_ident),)*
                    _ => Ok(Self::Custom(token)),
                }
            }

            /// Return a `str` representation of the header.
            ///
            /// The `str` returned will always be lower case.
            pub fn as_str(&self) -> &str {
                match self {
                    $(
                        HeaderFieldName::Registered(StaticFieldName::$variant) => $name,
                    )*
                    HeaderFieldName::Custom(s) => s.as_ref(),
                }
            }
        }

        #[test]
        fn derive_static_registered_field_names_from_bytes() {
            $(
                assert_eq!(Ok(HeaderFieldName::$static_ident), HeaderFieldName::from_bytes($name.as_bytes()));
            )*
        }
    };
}

// Standard field names as listed in [Hypertext Transfer Protocol (HTTP) Field Name
// Registry](https://www.iana.org/assignments/http-fields/http-fields.xhtml#field-names).
standard_field_name_impl! {
    /// Field name A-IM with a permanent status - reference RFC4229
    AIM, A_IM, "a-im",
    /// Field name Accept with a permanent status - reference RFC7231 Section 5.3.2
    Accept, ACCEPT, "accept",
    /// Field name Accept-Additions with a permanent status - reference RFC4229
    AcceptAdditions, ACCEPT_ADDITIONS, "accept-additions",
    /// Field name Accept-CH with a permanent status - reference RFC8942
    AcceptCH, ACCEPT_CH, "accept-ch",
    /// Field name Accept-Charset with a deprecated status - reference RFC7231 Section 5.3.3
    AcceptCharset, ACCEPT_CHARSET, "accept-charset",
    /// Field name Accept-Datetime with a permanent status - reference RFC7089
    AcceptDatetime, ACCEPT_DATETIME, "accept-datetime",
    /// Field name Accept-Encoding with a permanent status - reference RFC7231 Section 5.3.4
    AcceptEncoding, ACCEPT_ENCODING, "accept-encoding",
    /// Field name Accept-Features with a permanent status - reference RFC4229
    AcceptFeatures, ACCEPT_FEATURES, "accept-features",
    /// Field name Accept-Languages with a permanent status - reference RFC7231 Section 5.3.5
    AcceptLanguages, ACCEPT_LANGUAGES, "accept-languages",
    /// Field name Accept-Patch with a provisional status - reference RFC5789
    AcceptPatch, ACCEPT_PATCH, "accept-patch",
    /// Field name Accept-Post with a permanent status - reference W3C Linked Data Platform 1.0
    AcceptPost, ACCEPT_POST, "accept-post",
    /// Field name Accept-Ranges with a permanent status - reference RFC7233 Section 2.3
    AcceptRanges, ACCEPT_RANGES, "accept-ranges",
    /// Field name Access-Control-Allow-Credentials with a permanent status - reference fetch spec
    /// WHATWG
    AccessControlAllowCredentials, ACCESS_CONTROL_ALLOW_CREDENTIALS, "access-control-allow-credentials",
    /// Field name Access-Control-Allow-Headers with a permanent status - reference fetch spec
    /// WHATWG
    AccessControlAllowHeaders, ACCESS_CONTROL_ALLOW_HEADERS, "access-control-allow-headers",
    /// Field name Access-Control-Allow-Methods with a permanent status - reference fetch spec
    /// WHATWG
    AccessControlAllowMethods, ACCESS_CONTROL_ALLOW_METHODS, "access-control-allow-methods",
    /// Field name Access-Control-Allow-Origin with a permanent status - reference fetch spec
    /// WHATWG
    AccessControlAllowOrigin, ACCESS_CONTROL_ALLOW_ORIGIN, "access-control-allow-origin",
    /// Field name Access-Control-Expose-Headers with a permanent status - reference fetch spec
    /// WHATWG
    AccessControlExposeHeaders, ACCESS_CONTROL_EXPOSE_HEADERS, "access-control-expose-headers",
    /// Field name Access-Control-Max-Age with a permanent status - reference fetch spec
    /// WHATWG
    AccessControlMaxAge, ACCESS_CONTROL_MAX_AGE, "access-control-max-age",
    /// Field name Access-Control-Request-Headers with a permanent status - reference fetch spec
    /// WHATWG
    AccessControlRequestHeaders, ACCESS_CONTROL_REQUEST_HEADERS, "access-control-request-headers",
    /// Field name Access-Control-Request-Method with a permanent status - reference fetch spec
    /// WHATWG
    AccessControlRequestMethod, ACCESS_CONTROL_REQUEST_METHOD, "access-control-request-method",
    /// Field name Age with a permanent status - reference RFC7234 Section 5.1
    Age, AGE, "age",
    /// Field name Allow with a permanent status - reference RFC7231 Section 7.4.1
    Allow, ALLOW, "allow",
    /// Field name ALPN with a permanent status - reference RFC7639
    ALPN, ALPN, "alpn",
    /// Field name Alt-Svc with a permanent status - reference RFC7838
    AltSvc, ALT_SVC, "alt-svc",
    /// Field name Alt-Used with a permanent status - reference RFC7838
    AltUsed, ALT_USED, "alt-used",
    /// Field name Alternates with a permanent status - reference RFC4229
    Alternates, ALTERNATES, "alternates",
    /// Field name Apply-To-Redirect-Ref with a permanent status - reference RFC4437
    ApplyToRedirectRef, APPLY_TO_REDIRECT_REF, "apply-to-redirect-ref",
    /// Field name Authentication-Control with a permanent status - reference RFC8053
    AuthenticationControl, AUTHENTICATION_CONTROL, "authentication-control",
    /// Field name Authorization with a permanent status - reference RFC7235
    Authorization, AUTHORIZATION, "authorization",
    /// Field name C-Ext with a permanent status - reference RFC4229
    CExt, C_EXT, "c-ext",
    /// Field name C-Man with a permanent status - reference RFC4229
    CMan, C_MAN, "c-man",
    /// Field name C-Opt with a permanent status - reference RFC4229
    COpt, C_OPT, "c-opt",
    /// Field name C-PEP with a permanent status - reference RFC4229
    CPep, C_PEP, "c-pep",
    /// Field name C-PEP-Info with a deprecated status - reference RFC4229
    CPepInfo, C_PEP_INFO, "c-pep-info",
    /// Field name Cache-Control with a permanent status - reference RFC7234 Section 5.2
    CacheControl, CACHE_CONTROL, "cache-control",
    /// Field name Cal-Managed-ID with a permanent status - reference RFC8607
    CalManagedId, CAL_MANAGED_ID, "cal-managed-id",
    /// Field name CalDAV-Timezones with a permanent status - reference RFC7809
    CalDAVTimezones, CALDAV_TIMEZONES, "caldav-timezones",
    /// Field name CDN-Loop with a permanent status - reference RFC8586
    CDNLoop, CDN_LOOP, "cdn-loop",
    /// Field name Cert-Not-After with a permanent status - reference RFC8739
    CertNotAfter, CERT_NOT_AFTER, "cert-not-after",
    /// Field name Cert-Not-Before with a permanent status - reference RFC8739
    CertNotBefore, CERT_NOT_BEFORE, "cert-not-before",
    /// Field name Compliance with a provisional status - reference RFC4229
    Compliance, COMPLIANCE, "compliance",
    /// Field name Connection with a permanent status - reference RFC7230 Section 6.1
    Connection, CONNECTION, "connection",
    /// Field name Content-Disposition with a permanent status - reference RFC6266
    ContentDisposition, CONTENT_DISPOSITION, "content-disposition",
    /// Field name Content-Encoding with a permanent status - reference RFC7231 Section 3.1.2.2
    ContentEncoding, CONTENT_ENCODING, "content-encoding",
    /// Field name Content-ID with a permanent status - reference RFC4229
    ContentId, CONTENT_ID, "content-id",
    /// Field name Content-Language with a permanent status - reference RFC7231 Section 3.1.3.2
    ContentLanguage, CONTENT_LANGUAGE, "content-language",
    /// Field name Content-Length with a permanent status - reference RFC7230 Section 3.3.2
    ContentLength, CONTENT_LENGTH, "content-length",
    /// Field name Content-Location with a permanent status - reference RFC7231 Section 3.1.4.2
    ContentLocation, CONTENT_LOCATION, "content-location",
    /// Field name Content-Range with a permanent status - reference RFC7233 Section 4.2
    ContentRange, CONTENT_RANGE, "content-range",
    /// Field name Content-Script-Type with a permanent status - reference RFC4229
    ContentScriptType, CONTENT_SCRIPT_TYPE, "content-script-type",
    /// Field name Content-Style-Type with a permanent status - reference RFC4229
    ContentStyleType, CONTENT_STYLE_TYPE, "content-style-type",
    /// Field name Content-Transfer-Encoding with a permanent status - reference RFC4229
    ContentTransferEncoding, CONTENT_TRANSFER_ENCODING, "content-transfer-encoding",
    /// Field name Content-Type with a permanent status - reference RFC7231 Section 3.1.1.5
    ContentType, CONTENT_TYPE, "content-type",
    /// Field name Content-Version with a permanent status - reference RFC4229
    ContentVersion, CONTENT_VERSION, "content-version",
    /// Field name Cookie with a permanent status - reference RFC6265
    Cookie, COOKIE, "cookie",
    /// Field name Cost with a permanent status - reference RFC4229
    Cost, COST, "cost",
    /// Field name Cross-Origin-Resource-Policy with a permanent status - reference fetch spec
    /// WHATWG
    CrossOriginResourcePolicy, CROSS_ORIGIN_RESOURCE_POLICY, "cross-origin-resource-policy",
    /// Field name DASL with a permanent status - reference RFC5323
    DASL, DASL, "dasl",
    /// Field name Date with a permanent status - reference RFC7231 Section 7.1.1.2
    Date, DATE, "date",
    /// Field name DAV with a permanent status - reference RFC4918
    DAV, DAV, "dav",
    /// Field name Default-Style with a permanent status - reference RFC4229
    DefaultStyle, DEFAULT_STYLE, "default-style",
    /// Field name Delta-Base with a permanent status - reference RFC4229
    DeltaBase, DELTA_BASE, "delta-base",
    /// Field name Depth with a permanent status - reference RFC4918
    Depth, DEPTH, "depth",
    /// Field name Derived-From with a permanent status - reference RFC4229
    DerivedFrom, DERIVED_FROM, "derived-from",
    /// Field name Destination with a permanent status - reference RFC4918
    Destination, DESTINATION, "destination",
    /// Field name Differential-ID with a permanent status - reference RFC4229
    DifferentialId, DIFFERENTIAL_ID, "differential-id",
    /// Field name Digest with a permanent status - reference RFC4229
    Digest, DIGEST, "digest",
    /// Field name Early-Data with a permanent status - reference RFC8470
    EarlyData, EARLY_DATA, "early-data",
    /// Field name EDIINT-Features with a permanent status - reference RFC6017
    EDIINTFeatures, EDIINT_FEATURES, "ediint-features",
    /// Field name ETag with a permanent status - reference RFC7232 Section 2.3
    ETag, ETAG, "etag",
    /// Field name Expect with a permanent status - reference RFC7231 Section 5.1.1
    Expect, EXPECT, "expect",
    /// Field name Expires with a permanent status - reference RFC7234 Section 5.3
    Expires, EXPIRES, "expires",
    /// Field name Ext with a permanent status - reference RFC4229
    Ext, EXT, "ext",
    /// Field name Forwarded with a permanent status - reference RFC7239
    Forwarded, FORWARDED, "forwarded",
    /// Field name From with a permanent status - reference RFC7231 Section 5.5.1
    From, FROM, "from",
    /// Field name GetProfile with a permanent status - reference RFC4229
    GetProfile, GETPROFILE, "getprofile",
    /// Field name Hobareg with a permanent status - reference RFC7486
    Hobareg, HOBAREG, "hobareg",
    /// Field name Host with a permanent status - reference RFC7230 Section 5.4
    Host, HOST, "host",
    /// Field name HTTP2-Settings with a permanent status - reference RFC7540
    HTTP2Settings, HTTP2_SETTINGS, "http2-setting",
    /// Field name If with a permanent status - reference RFC4918
    If, IF, "if",
    /// Field name If-Match with a permanent status - reference RFC7232 Section 3.1
    IfMatch, IF_MATCH, "if-match",
    /// Field name If-Modified-Since with a permanent status - reference RFC7232 Section 3.3
    IfModifiedSince, IF_MODIFIED_SINCE, "if-modified-since",
    /// Field name If-None-Match with a permanent status - reference RFC7232 Section 3.2
    IfNoneMatch, IF_NONE_MATCH, "if-none-match",
    /// Field name If-Range with a permanent status - reference RFC7232 Section 3.5
    IfRange, IF_RANGE, "if-range",
    /// Field name If-Schedule-Tag-Match with a permanent status - reference RFC6638
    IfScheduleTagMatch, IF_SCHEDULE_TAG_MATCH, "if-schedule-tag-match",
    /// Field name If-Unmodified-Since with a permanent status - reference RFC7232 Section 3.4
    IfUnmodifiedSince, IF_UNMODIFIED_SINCE, "if-unmodified-since",
    /// Field name IM with a permanent status - reference RFC4229
    IM, IM, "im",
    /// Field name Include-Referred-Token-Binding-ID with a permanent status - reference RFC8473
    IncludeReferredTokenBindingId, INCLUDE_REFERRED_TOKEN_BINDING_ID, "include-referred-token-binding-id",
    /// Field name Keep-Alive with a permanent status - reference RFC4229
    KeepAlive, KEEP_ALICE, "keep-alive",
    /// Field name Label with a permanent status - reference RFC4229
    Label, LABEL, "label",
    /// Field name Last-Modified with a permanent status - reference RFC7232 Section 2.2
    LastModified, LAST_MODIFIED, "last-modified",
    /// Field name Link with a permanent status - reference RFC8288
    Link, LINK, "link",
    /// Field name Location with a permanent status - reference RFC7231 Section 7.1.2
    Location, LOCATION, "location",
    /// Field name Lock-Token with a permanent status - reference RFC4918
    LockToken, LOCK_TOKEN, "lock-token",
    /// Field name Man with a permanent status - reference RFC4229
    Man, MAN, "man",
    /// Field name Max-Forwards with a permanent status - reference RFC7231 Section 5.1.2
    MaxForwards, MAX_FORWARDS, "max-forwards",
    /// Field name Memento-Datetime with a permanent status - reference RFC7089
    MementoDatetime, MEMENTO_DATETIME, "memento-datetime",
    /// Field name Message-ID with a permanent status - reference RFC4229
    MessageId, MESSAGE_ID, "message-id",
    /// Field name Meter with a permanent status - reference RFC4229
    Meter, METER, "meter",
    /// Field name MIME-Version with a permanent status - reference RFC7231 Appendix A.1
    MIMEVersion, MIME_VERSION, "mime-version",
    /// Field name Negotiate with a permanent status - reference RFC4229
    Negotiate, NEGOTIATE, "negotiate",
    /// Field name Non-Compliance with a permanent status - reference RFC4229
    NonCompliance, NON_COMPLIANCE, "non-compliance",
    /// Field name Opt with a permanent status - reference RFC4229
    Opt, OPT, "opt",
    /// Field name Optional with a permanent status - reference RFC4229
    Optional, OPTIONAL, "optional",
    /// Field name Optional-WWW-Authenticate with a permanent status - reference RFC8053
    OptionalWWWAuthenticate, OPTIONAL_WWW_AUTHENTICATE, "optional-www-authenticate",
    /// Field name Ordering-Type with a permanent status - reference RFC4229
    OrderingType, ORDERING_TYPE, "ordering-type",
    /// Field name Origin with a permanent status - reference RFC6454
    Origin, ORIGIN, "origin",
    /// Field name OSCOR with a permanent status - reference RFC8613
    OSCOR, OSCOR, "oscor",
    /// Field name Overwrite with a permanent status - reference RFC4918
    Overwrite, OVERWRITE, "overwrite",
    /// Field name P3P with a permanent status - reference RFC4229
    P3P, P3P, "p3p",
    /// Field name PEP with a permanent status - reference RFC4229
    PEP, PEP, "pep",
    /// Field name Pep-Info with a permanent status - reference RFC4229
    PepInfo, PEP_INFO, "pep-info",
    /// Field name PICS-Label with a permanent status - reference RFC4229
    PICSLabel, PICS_LABEL, "pics-label",
    /// Field name Position with a permanent status - reference RFC4229
    Position, POSITION, "position",
    /// Field name Pragma with a permanent status - reference RFC7234 Section 5.4
    Pragma, PRAGME, "pragma",
    /// Field name Prefer with a permanent status - reference RFC7240
    Prefer, PREFER, "prefer",
    /// Field name Preference-Applied with a permanent status - reference RFC7240
    PreferenceApplied, PREFERENCE_APPLIED, "preference-applied",
    /// Field name ProfileObject with a permanent status - reference RFC4229
    ProfileObject, PROFILEOBJECT, "profileobject",
    /// Field name Protocol with a permanent status - reference RFC4229
    Protocol, PROTOCOL, "protocol",
    /// Field name Protocol-Request with a permanent status - reference RFC4229
    ProtocolRequest, PROTOCOL_REQUEST, "protocol-request",
    /// Field name Proxy-Authenticate with a permanent status - reference RFC7235 Section 4.3
    ProxyAuthenticate, PROXY_AUTHENTICATE, "proxy-authenticate",
    /// Field name Proxy-Authorization with a permanent status - reference RFC7235 Section 4.4
    ProxyAuthorization, PROXY_AUTHORIZATION, "proxy-authorization",
    /// Field name Proxy-Features with a permanent status - reference RFC4229
    ProxyFeatures, PROXY_FEATURES, "proxy-features",
    /// Field name Proxy-Instruction with a permanent status - reference RFC4229
    ProxyInstruction, PROXY_INSTRUCTION, "proxy-instruction",
    /// Field name Public with a permanent status - reference RFC4229
    Public, PUBLIC, "public",
    /// Field name Public-Key-Pins with a permanent status - reference RFC7469
    PublicKeyPins, PUBLIC_KEY_PINS, "public-key-pins",
    /// Field name Public-Key-Pins-Report-Only with a permanent status - reference RFC7469
    PublicKeyPinsReportOnly, PUBLIC_KEY_PINS_REPORT_ONLY, "public-key-pins-report-only",
    /// Field name Range with a permanent status - reference RFC7233 Section 3.1
    Range, RANGE, "range",
    /// Field name Redirect-Ref with a permanent status - reference RFC4437
    RedirectRef, REDIRECT_REF, "redirect-ref",
    /// Field name Referer with a permanent status - reference RFC7231 Section 5.5.2
    Referer, REFERER, "referer",
    /// Field name Replay-Nonce with a permanent status - reference RFC8555
    ReplayNonce, REPLAY_NONCE, "replay-nonce",
    /// Field name Resolution-Hint with a permanent status - reference RFC4229
    ResolutionHint, RESOLUTION_HINT, "resolution-hint",
    /// Field name Resolver-Location with a permanent status - reference RFC4229
    ResolverLocation, RESOLVER_LOCATION, "resolution-location",
    /// Field name Retry-After with a permanent status - reference RFC7231 Section 7.1.3
    RetryAfter, RETRY_AFTER, "retry-after",
    /// Field name Safe with a permanent status - reference RFC4229
    Safe, SAFE, "safe",
    /// Field name Schedule-Reply with a permanent status - reference RFC6638
    ScheduleReply, SCHEDULE_REPLAY, "schedule-replay",
    /// Field name Schedule-Tag with a permanent status - reference RFC6638
    ScheduleTag, SCHEDULE_TAG, "schedule-tag",
    /// Field name Sec-Token-Binding with a permanent status - reference RFC8473
    SecTokenBinding, SEC_TOKEN_BINDING, "sec-token-binding",
    /// Field name Sec-WebSocket-Accept with a permanent status - reference RFC6455
    SecWebsocketAccept, SEC_WEBSOCKET_ACCEPT, "sec-websocket-accept",
    /// Field name Sec-WebSocket-Extensions with a permanent status - reference RFC6455
    SecWebsocketExtensions, SEC_WEBSOCKET_EXTENSIONS, "sec-websocket-extensions",
    /// Field name Sec-WebSocket-Key with a permanent status - reference RFC6455
    SecWebsocketKey, SEC_WEBSOCKET_KEY, "sec-websocket-key",
    /// Field name Sec-WebSocket-Protocol with a permanent status - reference RFC6455
    SecWebsocketProtocol, SEC_WEBSOCKET_PROTOCOL, "sec-websocket-protocol",
    /// Field name Sec-WebSocket-Version with a permanent status - reference RFC6455
    SecWebsocketVersion, SEC_WEBSOCKET_VERSION, "sec-websocket-version",
    /// Field name Security-Scheme with a permanent status - reference RFC4229
    SecurityScheme, SECURITY_SCHEME, "security-scheme",
    /// Field name Server with a permanent status - reference RFC7231 Section 7.4.2
    Server, SERVER, "server",
    /// Field name Set-Cookie with a permanent status - reference RFC6265
    SetCookie, SET_COOKIE, "set-cookie",
    /// Field name SetProfile with a permanent status - reference RFC4229
    SetProfile, SETPROFILE, "setprofile",
    /// Field name SLUG with a permanent status - reference RFC5023
    SLUG, SLUG, "slug",
    /// Field name SoapAction with a permanent status - reference RFC4229
    SoapAction, SOAPACTION, "soapaction",
    /// Field name Status-URI with a permanent status - reference RFC4229
    StatusURI, STATUS_URI, "status-uri",
    /// Field name Strict-Transport-Security with a permanent status - reference RFC6797
    StrictTransportSecurity, STRICT_TRANSPORT_SECURITY, "strict-transport-security",
    /// Field name SubOK with a permanent status - reference RFC4229
    SubOk, SUBOK, "subok",
    /// Field name Subst with a permanent status - reference RFC4229
    Subst, SUBST, "subst",
    /// Field name Sunset with a permanent status - reference RFC8594
    Sunset, SUNSET, "sunset",
    /// Field name Surrogate-Capability with a permanent status - reference RFC4229
    SurrogateCapability, SURROGATE_CAPABILITY, "surrogate-capability",
    /// Field name Surrogate-Control with a permanent status - reference RFC4229
    SurrogateControl, SURROGATE_CONTROL, "surrogate-control",
    /// Field name TCN with a permanent status - reference RFC4229
    TCN, TCN, "tcn",
    /// Field name TE with a permanent status - reference RFC7230 Section 4.3
    Te, TE, "te",
    /// Field name Timeout with a permanent status - reference RFC4918
    Timeout, TIMEOUT, "timeout",
    /// Field name Title with a permanent status - reference RFC4229
    Title, TITLE, "title",
    /// Field name Topic with a permanent status - reference RFC8030
    Topic, TOPIC, "topic",
    /// Field name Trailer with a permanent status - reference RFC7230 Section 4.4
    Trailer, TRAILER, "trailer",
    /// Field name Transfer-Encoding with a permanent status - reference RFC7230 Section 3.3.1
    TransferEncoding, TRANSFER_ENCODING, "transfer-encoding",
    /// Field name TTL with a permanent status - reference RFC8030
    TTL, TTL, "ttl",
    /// Field name UA-Color with a permanent status - reference RFC4229
    UaColor, UA_COLOR, "ua-color",
    /// Field name UA-Media with a permanent status - reference RFC4229
    UaMedia, UA_MEDIA, "ua-media",
    /// Field name UA-Pixels with a permanent status - reference RFC4229
    UaPixels, UA_PIXELS, "ua-pixels",
    /// Field name UA-Resolution with a permanent status - reference RFC4229
    UaResolution, UA_RESOLUTION, "ua-resolution",
    /// Field name UA-Windowpixels with a permanent status - reference RFC4229
    UaWindowpixels, UA_WINDOWPIXELS, "ua-windowpixels",
    /// Field name Upgrade with a permanent status - reference RFC7230 Section 6.7
    Upgrade, UPGRADE, "upgrade",
    /// Field name Urgency with a permanent status - reference RFC8030
    Urgency, URGENCY, "urgency",
    /// Field name URI with a permanent status - reference RFC4229
    URI, URI, "uri",
    /// Field name User-Agent with a permanent status - reference RFC7231 Section 5.5.3
    UserAgent, USER_AGENT, "user-agent",
    /// Field name Vary-Variant with a permanent status - reference RFC4229
    VaryVariant, VARY_VARIANT, "vary-variant",
    /// Field name Vary with a permanent status - reference RFC7231 Section 7.1.4
    Vary, VARY, "vary",
    /// Field name Version with a permanent status - reference RFC4229
    Version, VERSION, "version",
    /// Field name Via with a permanent status - reference RFC7230 Section 5.7.1
    Via, VIA, "via",
    /// Field name Want-Digest with a permanent status - reference RFC4229
    WantDigest, WANT_DIGEST, "want-digest",
    /// Field name Warning with a permanent status - reference RFC7234 Section 5.5
    Warning, WARNING, "warning",
    /// Field name WWW-Authenticate with a permanent status - reference RFC7235 Section 4.1
    WWWAuthenticate, WWW_AUTHENTICATE, "www-authenticate",
    /// Field name X-Content-Type-Options with a permanent status - reference fetch spec
    /// WHATWG
    XContentTypeOptions, X_CONTENT_TYPE_OPTIONS, "x-content-type-options",
    /// Field name X-Frame-Options with a permanent status - reference RFC7034
    XFrameOptions, X_FRAME_OPTIONS, "x-frame-options",
}
