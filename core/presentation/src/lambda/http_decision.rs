//! # HTTPルーティング・エラー応答決定コア（形式検証対象）
//!
//! Presentation層における純粋なプロトコル決定ロジック（HTTPルーティングおよび
//! エラーからHTTPステータスコード・公開メッセージへのサニタイズ変換）を、
//! AWS SDK・Tokio・SeaORM等の外部クレートから完全に独立した Verified Core として提供する。
//!
//! ## 形式検証（Verus + Kani）
//!
//! | ツール | 主な証明内容 |
//! |---|---|
//! | **Verus**（`--no-cheating`） | 任意長 `(method, path)` に対する CORS プリフライト（`OPTIONS`）最優先性、エンドポイント分離、単数/複数パス混同不在、HTTPステータス境界、インフラ詳細エラーの非漏洩（不可識別性） |
//! | **Kani** | `bytes_eq` と標準スライス比較の一致、`route_method_and_path` と標準文字列パターンマッチの同値性、公開エラーメッセージ文字列の検証 |
//!
//! ## フロー
//!
//! ```text
//! `router::route`
//!   └─▶ `route_method_and_path` / `route_bytes`（Verus + Kani 検証済）
//!
//! `inquiry_handler::handler` / `create_inquiry`
//!   ├─▶ `static_route_status_code`（Verus + Kani 検証済）
//!   └─▶ `decide_public_error`（Verus + Kani 検証済：インフラエラーを500へ秘匿化）
//! ```

#![cfg_attr(any(kani, verus_keep_ghost), no_std)]
#![cfg_attr(kani, allow(unexpected_cfgs))]

#[cfg(verus_keep_ghost)]
use vstd::prelude::*;

/// ルート定義
///
/// | バリアント | 説明 |
/// |---|---|
/// | `Options` | `OPTIONS *`（CORSプリフライト） |
/// | `CreateInquiry` | `POST /api/v1/inquiry` |
/// | `FindInquiries` | `GET /api/v1/inquiries` |
/// | `NotFound` | 404 Not Found |
/// | `MethodNotAllowed` | 405 Method Not Allowed |
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    /// OPTIONS（CORSプリフライト）
    Options,
    /// POST /api/v1/inquiry - 問い合わせ作成
    CreateInquiry,
    /// GET /api/v1/inquiries - 問い合わせ一覧（未実装）
    FindInquiries,
    /// 404 Not Found
    NotFound,
    /// 405 Method Not Allowed
    MethodNotAllowed,
}

/// ユースケース失敗時の上位エラー分類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsecaseErrorCategory {
    /// ドメインバリデーションエラー（400）
    DomainValidation,
    /// 重複エラー（409）
    DuplicateConflict,
    /// リポジトリ内部エラー：NotFound / Infrastructure / CorruptedData（500へ秘匿化）
    RepositoryInternal,
}

/// リポジトリ層エラー種別（内部詳細漏洩防止の証明用モデル）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepositoryFailureKind {
    /// 重複エラー
    Duplicate,
    /// 見つからないエラー
    NotFound,
    /// インフラ障害エラー
    Infrastructure,
    /// 保存データ破損エラー
    CorruptedData,
}

/// クライアントへ返却可能なサニタイズ済みエラーメッセージ種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublicErrorMessage {
    /// `"invalid request"`
    InvalidRequest,
    /// `"inquiry already exists"`
    InquiryAlreadyExists,
    /// `"internal server error"`
    InternalServerError,
}

impl PublicErrorMessage {
    /// サニタイズ済みの固定エラーメッセージ文字列を返す
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidRequest => "invalid request",
            Self::InquiryAlreadyExists => "inquiry already exists",
            Self::InternalServerError => "internal server error",
        }
    }
}

#[cfg(verus_keep_ghost)]
verus! {

#[verifier::external_type_specification]
pub struct ExRoute(Route);

#[verifier::external_type_specification]
pub struct ExUsecaseErrorCategory(UsecaseErrorCategory);

#[verifier::external_type_specification]
pub struct ExRepositoryFailureKind(RepositoryFailureKind);

#[verifier::external_type_specification]
pub struct ExPublicErrorMessage(PublicErrorMessage);

/// `"OPTIONS"` のバイト列仕様
pub open spec fn options_method_spec() -> Seq<u8> {
    seq![b'O', b'P', b'T', b'I', b'O', b'N', b'S']
}

/// `"POST"` のバイト列仕様
pub open spec fn post_method_spec() -> Seq<u8> {
    seq![b'P', b'O', b'S', b'T']
}

/// `"GET"` のバイト列仕様
pub open spec fn get_method_spec() -> Seq<u8> {
    seq![b'G', b'E', b'T']
}

/// `"/api/v1/inquiry"` のバイト列仕様
pub open spec fn inquiry_path_spec() -> Seq<u8> {
    seq![
        b'/', b'a', b'p', b'i', b'/', b'v', b'1', b'/',
        b'i', b'n', b'q', b'u', b'i', b'r', b'y',
    ]
}

/// `"/api/v1/inquiries"` のバイト列仕様
pub open spec fn inquiries_path_spec() -> Seq<u8> {
    seq![
        b'/', b'a', b'p', b'i', b'/', b'v', b'1', b'/',
        b'i', b'n', b'q', b'u', b'i', b'r', b'i', b'e', b's',
    ]
}

/// 許可されたHTTPステータスコード集合の仕様
pub open spec fn is_allowed_http_status_spec(status: i64) -> bool {
    status == 200
        || status == 201
        || status == 400
        || status == 404
        || status == 405
        || status == 409
        || status == 500
        || status == 501
}

/// 内部リポジトリエラー（外部へ詳細を漏洩してはならない分類）の仕様
pub open spec fn is_sensitive_repository_failure_spec(kind: RepositoryFailureKind) -> bool {
    match kind {
        RepositoryFailureKind::Duplicate => false,
        RepositoryFailureKind::NotFound => true,
        RepositoryFailureKind::Infrastructure => true,
        RepositoryFailureKind::CorruptedData => true,
    }
}

/// リポジトリ失敗種別の正規化仕様
pub open spec fn classify_repository_failure_spec(kind: RepositoryFailureKind) -> UsecaseErrorCategory {
    match kind {
        RepositoryFailureKind::Duplicate => UsecaseErrorCategory::DuplicateConflict,
        RepositoryFailureKind::NotFound => UsecaseErrorCategory::RepositoryInternal,
        RepositoryFailureKind::Infrastructure => UsecaseErrorCategory::RepositoryInternal,
        RepositoryFailureKind::CorruptedData => UsecaseErrorCategory::RepositoryInternal,
    }
}

/// 公開エラー応答決定の仕様
pub open spec fn decide_public_error_spec(category: UsecaseErrorCategory) -> (i64, PublicErrorMessage) {
    match category {
        UsecaseErrorCategory::DomainValidation => (400i64, PublicErrorMessage::InvalidRequest),
        UsecaseErrorCategory::DuplicateConflict => (409i64, PublicErrorMessage::InquiryAlreadyExists),
        UsecaseErrorCategory::RepositoryInternal => (500i64, PublicErrorMessage::InternalServerError),
    }
}

}

/// 2つのバイトスライスが完全に等しいか判定する
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec(res =>
    ensures
        res <==> (lhs@ == rhs@),
))]
pub fn bytes_eq(lhs: &[u8], rhs: &[u8]) -> bool {
    if lhs.len() != rhs.len() {
        return false;
    }

    let mut index: usize = 0;
    while index < lhs.len() {
        #[cfg(verus_keep_ghost)]
        verus_builtin_macros::verus_proof_expr!({
            invariant([
                lhs.len() == rhs.len(),
                0 <= index <= lhs.len(),
                forall|k: int| 0 <= k < index ==> lhs@[k] == rhs@[k],
            ]);
            decreases(lhs.len() - index);
        });

        if lhs[index] != rhs[index] {
            return false;
        }
        index += 1;
    }

    #[cfg(verus_keep_ghost)]
    verus_builtin_macros::verus_proof_expr!({
        proof! {
            assert(lhs@ =~= rhs@);
        }
    });

    true
}

/// メソッドが `"OPTIONS"` であるか判定する
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec(res =>
    ensures
        res <==> (method@ == options_method_spec()),
))]
pub fn is_options_method(method: &[u8]) -> bool {
    let matched: bool = method.len() == 7
        && method[0] == b'O'
        && method[1] == b'P'
        && method[2] == b'T'
        && method[3] == b'I'
        && method[4] == b'O'
        && method[5] == b'N'
        && method[6] == b'S';

    #[cfg(verus_keep_ghost)]
    verus_builtin_macros::verus_proof_expr!({
        proof! {
            if matched {
                assert(method@ =~= options_method_spec());
            }
        }
    });

    matched
}

/// メソッドが `"POST"` であるか判定する
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec(res =>
    ensures
        res <==> (method@ == post_method_spec()),
))]
pub fn is_post_method(method: &[u8]) -> bool {
    let matched: bool = method.len() == 4
        && method[0] == b'P'
        && method[1] == b'O'
        && method[2] == b'S'
        && method[3] == b'T';

    #[cfg(verus_keep_ghost)]
    verus_builtin_macros::verus_proof_expr!({
        proof! {
            if matched {
                assert(method@ =~= post_method_spec());
            }
        }
    });

    matched
}

/// メソッドが `"GET"` であるか判定する
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec(res =>
    ensures
        res <==> (method@ == get_method_spec()),
))]
pub fn is_get_method(method: &[u8]) -> bool {
    let matched: bool =
        method.len() == 3 && method[0] == b'G' && method[1] == b'E' && method[2] == b'T';

    #[cfg(verus_keep_ghost)]
    verus_builtin_macros::verus_proof_expr!({
        proof! {
            if matched {
                assert(method@ =~= get_method_spec());
            }
        }
    });

    matched
}

/// パスが `"/api/v1/inquiry"` であるか判定する
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec(res =>
    ensures
        res <==> (path@ == inquiry_path_spec()),
))]
pub fn is_inquiry_path(path: &[u8]) -> bool {
    let matched: bool = path.len() == 15
        && path[0] == b'/'
        && path[1] == b'a'
        && path[2] == b'p'
        && path[3] == b'i'
        && path[4] == b'/'
        && path[5] == b'v'
        && path[6] == b'1'
        && path[7] == b'/'
        && path[8] == b'i'
        && path[9] == b'n'
        && path[10] == b'q'
        && path[11] == b'u'
        && path[12] == b'i'
        && path[13] == b'r'
        && path[14] == b'y';

    #[cfg(verus_keep_ghost)]
    verus_builtin_macros::verus_proof_expr!({
        proof! {
            if matched {
                assert(path@ =~= inquiry_path_spec());
            }
        }
    });

    matched
}

/// パスが `"/api/v1/inquiries"` であるか判定する
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec(res =>
    ensures
        res <==> (path@ == inquiries_path_spec()),
))]
pub fn is_inquiries_path(path: &[u8]) -> bool {
    let matched: bool = path.len() == 17
        && path[0] == b'/'
        && path[1] == b'a'
        && path[2] == b'p'
        && path[3] == b'i'
        && path[4] == b'/'
        && path[5] == b'v'
        && path[6] == b'1'
        && path[7] == b'/'
        && path[8] == b'i'
        && path[9] == b'n'
        && path[10] == b'q'
        && path[11] == b'u'
        && path[12] == b'i'
        && path[13] == b'r'
        && path[14] == b'i'
        && path[15] == b'e'
        && path[16] == b's';

    #[cfg(verus_keep_ghost)]
    verus_builtin_macros::verus_proof_expr!({
        proof! {
            if matched {
                assert(path@ =~= inquiries_path_spec());
            }
        }
    });

    matched
}

/// HTTPメソッドとパスのバイト列からルートを決定する
///
/// ## 優先順位と不変条件
///
/// 1. `OPTIONS` はパスによらず常に `Route::Options`（CORSプリフライト最優先）
/// 2. `POST /api/v1/inquiry` のみ `Route::CreateInquiry`
/// 3. `GET /api/v1/inquiries` のみ `Route::FindInquiries`
/// 4. 既知パスに対する上記以外のメソッドは `Route::MethodNotAllowed`（405）
/// 5. それ以外はすべて `Route::NotFound`（404）
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec(res =>
    ensures
        res == Route::Options <==> (method@ == options_method_spec()),
        res == Route::CreateInquiry
            <==> (method@ == post_method_spec() && path@ == inquiry_path_spec()),
        res == Route::FindInquiries
            <==> (method@ == get_method_spec() && path@ == inquiries_path_spec()),
        res == Route::MethodNotAllowed <==> (
            method@ != options_method_spec()
                && (
                    (path@ == inquiry_path_spec() && method@ != post_method_spec())
                        || (path@ == inquiries_path_spec() && method@ != get_method_spec())
                )
        ),
        res == Route::NotFound <==> (
            method@ != options_method_spec()
                && path@ != inquiry_path_spec()
                && path@ != inquiries_path_spec()
        ),
))]
pub fn route_bytes(method: &[u8], path: &[u8]) -> Route {
    if is_options_method(method) {
        return Route::Options;
    }

    let on_inquiry: bool = is_inquiry_path(path);
    if on_inquiry {
        if is_post_method(method) {
            return Route::CreateInquiry;
        }
        return Route::MethodNotAllowed;
    }

    let on_inquiries: bool = is_inquiries_path(path);
    if on_inquiries {
        if is_get_method(method) {
            return Route::FindInquiries;
        }
        return Route::MethodNotAllowed;
    }

    Route::NotFound
}

/// HTTPメソッドとパス文字列からルートを決定する
pub fn route_method_and_path(method: &str, path: &str) -> Route {
    route_bytes(method.as_bytes(), path.as_bytes())
}

/// 静的ルート（ユースケース実行を伴わないルート）のHTTPステータスコードを返す
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec(res =>
    ensures
        route == Route::CreateInquiry <==> res.is_none(),
        route == Route::Options <==> (res == Some(200i64)),
        route == Route::FindInquiries <==> (res == Some(501i64)),
        route == Route::NotFound <==> (res == Some(404i64)),
        route == Route::MethodNotAllowed <==> (res == Some(405i64)),
        match res {
            Some(code) => is_allowed_http_status_spec(code),
            None => true,
        },
))]
pub const fn static_route_status_code(route: Route) -> Option<i64> {
    match route {
        Route::CreateInquiry => None,
        Route::Options => Some(200),
        Route::FindInquiries => Some(501),
        Route::NotFound => Some(404),
        Route::MethodNotAllowed => Some(405),
    }
}

/// リポジトリ失敗種別をユースケースエラー分類へ正規化する
///
/// `NotFound` / `Infrastructure` / `CorruptedData` はすべて単一の
/// `UsecaseErrorCategory::RepositoryInternal` に畳み込み、内部状態を外部へ識別させない。
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec(res =>
    ensures
        res == classify_repository_failure_spec(kind),
        kind == RepositoryFailureKind::Duplicate
            <==> (res == UsecaseErrorCategory::DuplicateConflict),
        is_sensitive_repository_failure_spec(kind)
            <==> (res == UsecaseErrorCategory::RepositoryInternal),
))]
pub const fn classify_repository_failure(kind: RepositoryFailureKind) -> UsecaseErrorCategory {
    match kind {
        RepositoryFailureKind::Duplicate => UsecaseErrorCategory::DuplicateConflict,
        RepositoryFailureKind::NotFound
        | RepositoryFailureKind::Infrastructure
        | RepositoryFailureKind::CorruptedData => UsecaseErrorCategory::RepositoryInternal,
    }
}

/// エラー分類からクライアント返却用の `(HTTPステータスコード, サニタイズ済みメッセージ種別)` を決定する
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec(res =>
    ensures
        res == decide_public_error_spec(category),
        is_allowed_http_status_spec(res.0),
        400 <= res.0 && res.0 <= 599,
        category == UsecaseErrorCategory::DomainValidation
            <==> (res.0 == 400 && res.1 == PublicErrorMessage::InvalidRequest),
        category == UsecaseErrorCategory::DuplicateConflict
            <==> (res.0 == 409 && res.1 == PublicErrorMessage::InquiryAlreadyExists),
        category == UsecaseErrorCategory::RepositoryInternal
            <==> (res.0 == 500 && res.1 == PublicErrorMessage::InternalServerError),
))]
pub const fn decide_public_error(category: UsecaseErrorCategory) -> (i64, PublicErrorMessage) {
    match category {
        UsecaseErrorCategory::DomainValidation => (400, PublicErrorMessage::InvalidRequest),
        UsecaseErrorCategory::DuplicateConflict => (409, PublicErrorMessage::InquiryAlreadyExists),
        UsecaseErrorCategory::RepositoryInternal => (500, PublicErrorMessage::InternalServerError),
    }
}

#[cfg(verus_keep_ghost)]
verus! {

/// 補題: 単数形 `/api/v1/inquiry`（15バイト）と複数形 `/api/v1/inquiries`（17バイト）は決して混同されない
pub proof fn lemma_inquiry_paths_distinct()
    ensures
        inquiry_path_spec() != inquiries_path_spec(),
{
    assert(inquiry_path_spec().len() == 15);
    assert(inquiries_path_spec().len() == 17);
}

/// 補題: 秘匿すべきリポジトリ内部エラー（`NotFound`, `Infrastructure`, `CorruptedData`）は
/// いずれも外部応答で完全に同一の `(500, InternalServerError)` に写像され、内部原因を漏洩しない（非干渉性）
pub proof fn lemma_sensitive_repository_errors_indistinguishable(
    k1: RepositoryFailureKind,
    k2: RepositoryFailureKind,
)
    requires
        is_sensitive_repository_failure_spec(k1),
        is_sensitive_repository_failure_spec(k2),
    ensures
        decide_public_error_spec(classify_repository_failure_spec(k1))
            == decide_public_error_spec(classify_repository_failure_spec(k2)),
        decide_public_error_spec(classify_repository_failure_spec(k1))
            == (500i64, PublicErrorMessage::InternalServerError),
{
}

}

#[cfg(kani)]
mod kani_proofs {
    use super::*;

    /// 証明: `bytes_eq` が任意のバイトスライス（最大 6 バイト）において標準の `==` と完全一致すること
    #[kani::proof]
    #[kani::unwind(8)]
    fn proof_bytes_eq_matches_core_slice_eq() {
        let lhs_buf: [u8; 6] = kani::any();
        let rhs_buf: [u8; 6] = kani::any();
        let lhs_len: usize = kani::any();
        let rhs_len: usize = kani::any();
        kani::assume(lhs_len <= lhs_buf.len());
        kani::assume(rhs_len <= rhs_buf.len());

        let lhs: &[u8] = &lhs_buf[..lhs_len];
        let rhs: &[u8] = &rhs_buf[..rhs_len];
        assert_eq!(bytes_eq(lhs, rhs), lhs == rhs);
    }

    /// 証明: 既知メソッド・パス・近似パス・空文字を含む全組合せで
    /// `route_method_and_path` が仕様の文字列パターンマッチと完全一致すること
    #[kani::proof]
    #[kani::unwind(20)]
    fn proof_route_method_and_path_matches_specification() {
        let method_idx: u8 = kani::any();
        kani::assume(method_idx < 6);
        let method: &str = match method_idx {
            0 => "OPTIONS",
            1 => "POST",
            2 => "GET",
            3 => "PUT",
            4 => "DELETE",
            _ => "",
        };

        let path_idx: u8 = kani::any();
        kani::assume(path_idx < 6);
        let path: &str = match path_idx {
            0 => "/api/v1/inquiry",
            1 => "/api/v1/inquiries",
            2 => "/api/v1/inquiry/",
            3 => "/api/v1/inquirie",
            4 => "/",
            _ => "",
        };

        let actual: Route = route_method_and_path(method, path);
        let expected: Route = match (method, path) {
            ("OPTIONS", _) => Route::Options,
            ("POST", "/api/v1/inquiry") => Route::CreateInquiry,
            ("GET", "/api/v1/inquiries") => Route::FindInquiries,
            (_, "/api/v1/inquiry") | (_, "/api/v1/inquiries") => Route::MethodNotAllowed,
            _ => Route::NotFound,
        };

        assert_eq!(actual, expected);
    }

    /// 証明: 任意の生バイトパス（最大 20 バイト）に対して `"OPTIONS"` メソッドは
    /// 常に `Route::Options`（HTTP 200）にルーティングされること
    #[kani::proof]
    fn proof_options_preflight_always_wins_on_arbitrary_path() {
        let path_buf: [u8; 20] = kani::any();
        let path_len: usize = kani::any();
        kani::assume(path_len <= path_buf.len());

        let routed: Route = route_bytes(b"OPTIONS", &path_buf[..path_len]);
        assert_eq!(routed, Route::Options);
        assert_eq!(static_route_status_code(routed), Some(200));
    }

    /// 証明: インフラ・データ破損・未検出エラーはすべて同一の `500 internal server error` に
    /// サニタイズされ、内部詳細文字列が外部へ漏洩しないこと
    #[kani::proof]
    fn proof_error_sanitization_and_status_codes() {
        let domain_res: (i64, PublicErrorMessage) =
            decide_public_error(UsecaseErrorCategory::DomainValidation);
        assert_eq!(domain_res.0, 400);
        assert_eq!(domain_res.1.as_str(), "invalid request");

        let dup_res: (i64, PublicErrorMessage) = decide_public_error(classify_repository_failure(
            RepositoryFailureKind::Duplicate,
        ));
        assert_eq!(dup_res.0, 409);
        assert_eq!(dup_res.1.as_str(), "inquiry already exists");

        let not_found_res: (i64, PublicErrorMessage) =
            decide_public_error(classify_repository_failure(RepositoryFailureKind::NotFound));
        let infra_res: (i64, PublicErrorMessage) = decide_public_error(
            classify_repository_failure(RepositoryFailureKind::Infrastructure),
        );
        let corrupted_res: (i64, PublicErrorMessage) = decide_public_error(
            classify_repository_failure(RepositoryFailureKind::CorruptedData),
        );

        assert_eq!(not_found_res, infra_res);
        assert_eq!(infra_res, corrupted_res);
        assert_eq!(infra_res.0, 500);
        assert_eq!(infra_res.1.as_str(), "internal server error");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optionsメソッドは任意のパスでoptionsルートかつ200になる() {
        let r1: Route = route_method_and_path("OPTIONS", "/api/v1/inquiry");
        let r2: Route = route_method_and_path("OPTIONS", "/unknown/path");
        assert_eq!(r1, Route::Options);
        assert_eq!(r2, Route::Options);
        assert_eq!(static_route_status_code(r1), Some(200));
    }

    #[test]
    fn 内部リポジトリエラー3種はすべて同一の500レスポンスに秘匿される() {
        let nf: (i64, PublicErrorMessage) =
            decide_public_error(classify_repository_failure(RepositoryFailureKind::NotFound));
        let infra: (i64, PublicErrorMessage) = decide_public_error(classify_repository_failure(
            RepositoryFailureKind::Infrastructure,
        ));
        let corrupted: (i64, PublicErrorMessage) = decide_public_error(
            classify_repository_failure(RepositoryFailureKind::CorruptedData),
        );

        assert_eq!(nf, (500, PublicErrorMessage::InternalServerError));
        assert_eq!(nf, infra);
        assert_eq!(infra, corrupted);
    }

    #[test]
    fn ドメインエラーは400かつ重複エラーは409に変換される() {
        let domain_err: (i64, PublicErrorMessage) =
            decide_public_error(UsecaseErrorCategory::DomainValidation);
        let dup_err: (i64, PublicErrorMessage) = decide_public_error(classify_repository_failure(
            RepositoryFailureKind::Duplicate,
        ));

        assert_eq!(domain_err, (400, PublicErrorMessage::InvalidRequest));
        assert_eq!(dup_err, (409, PublicErrorMessage::InquiryAlreadyExists));
    }
}
