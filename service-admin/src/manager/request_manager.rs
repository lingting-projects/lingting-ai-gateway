use std::collections::HashMap;

use anyhow::Result;
use framework_core::types::{PaginationParams, PaginationResult};
use types_admin::dto::{
    RequestMainDetailVO, RequestMainQO, RequestMainVO, RequestSubVO,
};

use crate::service::RequestService;

/// 请求日志跨表编排：主日志分页并嵌入各自子日志。
pub struct RequestManager;

impl RequestManager {
    /// 主请求日志分页，附带每条主日志下的子请求日志。
    ///
    /// 先查主日志拿到主键集合，再一次性查出全部子日志后按主键归组，避免逐条查询。
    pub async fn page_detail(
        pagination: &PaginationParams,
        conditions: &RequestMainQO,
    ) -> Result<PaginationResult<RequestMainDetailVO>> {
        let page = RequestService::main_page(pagination, conditions).await?;
        let ids: Vec<i64> = page.records.iter().map(|main| main.id).collect();

        let subs = if ids.is_empty() {
            Vec::new()
        } else {
            RequestService::find_sub_by_mains(&ids).await?
        };

        let mut grouped: HashMap<i64, Vec<RequestSubVO>> = HashMap::new();
        for sub in subs {
            grouped
                .entry(sub.main_request_id)
                .or_default()
                .push(RequestSubVO::from(sub));
        }

        let records = page
            .records
            .into_iter()
            .map(|main| {
                let children = grouped.remove(&main.id).unwrap_or_default();
                RequestMainDetailVO {
                    main: RequestMainVO::from(main),
                    children,
                }
            })
            .collect();

        Ok(PaginationResult {
            total: page.total,
            records,
        })
    }
}
