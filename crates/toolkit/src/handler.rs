use crate::tools::ToolkitTools;
use async_trait::async_trait;
use rust_mcp_sdk::{
    McpServer, RequestContext,
    mcp_server::ServerHandler,
    schema::{
        CallToolRequestParams, ListToolsResult, ListToolsResultCacheScope, PaginatedRequestParams,
        RpcError, ServerResult, schema_utils::CallToolError,
    },
};
use std::sync::Arc;

/// MCP handler shared by both server transports.
#[derive(Debug, Default, Clone, Copy)]
pub struct ToolkitHandler;

#[async_trait]
impl ServerHandler for ToolkitHandler {
    async fn handle_list_tools_request(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: &RequestContext,
        _runtime: Arc<dyn McpServer>,
    ) -> Result<ListToolsResult, RpcError> {
        let mut tools = ToolkitTools::tools();
        crate::tools::apply_output_schemas(&mut tools);

        Ok(ListToolsResult {
            tools,
            meta: None,
            next_cursor: None,
            cache_scope: ListToolsResultCacheScope::Private,
            result_type: "complete".to_string(),
            ttl_ms: 0,
        })
    }

    async fn handle_call_tool_request(
        &self,
        params: CallToolRequestParams,
        _context: &RequestContext,
        _runtime: Arc<dyn McpServer>,
    ) -> Result<ServerResult, CallToolError> {
        let tool = ToolkitTools::try_from(params).map_err(CallToolError::new)?;

        let result = match tool {
            ToolkitTools::GenerateUuidTool(tool) => tool.call_tool()?,
            ToolkitTools::RandomNumberTool(tool) => tool.call_tool()?,
            ToolkitTools::RandomStringTool(tool) => tool.call_tool()?,
            ToolkitTools::RandomPrimeTool(tool) => tool.call_tool()?,
            ToolkitTools::CurrentTimeTool(tool) => tool.call_tool()?,
            ToolkitTools::UnixToIso8601Tool(tool) => tool.call_tool()?,
            ToolkitTools::Iso8601ToUnixTool(tool) => tool.call_tool()?,
            ToolkitTools::SolarToLunarDateTool(tool) => tool.call_tool()?,
            ToolkitTools::LunarToSolarDateTool(tool) => tool.call_tool()?,
            ToolkitTools::CronValidateTool(tool) => tool.call_tool()?,
            ToolkitTools::CronNextTool(tool) => tool.call_tool()?,
            ToolkitTools::CronGenerateTool(tool) => tool.call_tool()?,
            ToolkitTools::CronParseTool(tool) => tool.call_tool()?,
            ToolkitTools::CronPrevTool(tool) => tool.call_tool()?,
            ToolkitTools::UrlEncodeTool(tool) => tool.call_tool()?,
            ToolkitTools::UrlDecodeTool(tool) => tool.call_tool()?,
            ToolkitTools::BinaryToHexTool(tool) => tool.call_tool()?,
            ToolkitTools::HexToBinaryTool(tool) => tool.call_tool()?,
            ToolkitTools::BinaryToStringTool(tool) => tool.call_tool()?,
            ToolkitTools::StringToBinaryTool(tool) => tool.call_tool()?,
            ToolkitTools::ChangeCaseTool(tool) => tool.call_tool()?,
            ToolkitTools::NamingCaseTool(tool) => tool.call_tool()?,
            ToolkitTools::JwtDecodeTool(tool) => tool.call_tool()?,
            ToolkitTools::CidrNetworkTool(tool) => tool.call_tool()?,
            ToolkitTools::RegexMatchTool(tool) => tool.call_tool()?,
            ToolkitTools::RegexReplaceTool(tool) => tool.call_tool()?,
            ToolkitTools::RegexTestTool(tool) => tool.call_tool()?,
            ToolkitTools::JsonPrettyTool(tool) => tool.call_tool()?,
            ToolkitTools::JsonMinifyTool(tool) => tool.call_tool()?,
            ToolkitTools::JsonQueryTool(tool) => tool.call_tool()?,
            ToolkitTools::JsonValidateTool(tool) => tool.call_tool()?,
            ToolkitTools::JsonSchemaValidateTool(tool) => tool.call_tool()?,
            ToolkitTools::DigestTool(tool) => tool.call_tool()?,
            ToolkitTools::HmacTool(tool) => tool.call_tool()?,
            ToolkitTools::RsaEncryptTool(tool) => tool.call_tool()?,
            ToolkitTools::RsaDecryptTool(tool) => tool.call_tool()?,
            ToolkitTools::RsaSignTool(tool) => tool.call_tool()?,
            ToolkitTools::RsaVerifyTool(tool) => tool.call_tool()?,
            ToolkitTools::RsaGenerateKeyTool(tool) => tool.call_tool_async().await?,
            ToolkitTools::RsaPrivateToPublicTool(tool) => tool.call_tool()?,
            ToolkitTools::RsaConvertPrivateKeyTool(tool) => tool.call_tool()?,
            ToolkitTools::RsaKeyComponentsTool(tool) => tool.call_tool()?,
            ToolkitTools::AesEncryptTool(tool) => tool.call_tool()?,
            ToolkitTools::AesDecryptTool(tool) => tool.call_tool()?,
            ToolkitTools::AesGenerateKeyTool(tool) => tool.call_tool()?,
            ToolkitTools::AesGenerateIvTool(tool) => tool.call_tool()?,
            ToolkitTools::DesEncryptTool(tool) => tool.call_tool()?,
            ToolkitTools::DesDecryptTool(tool) => tool.call_tool()?,
            ToolkitTools::DesGenerateKeyTool(tool) => tool.call_tool()?,
            ToolkitTools::DesGenerateIvTool(tool) => tool.call_tool()?,
        };

        Ok(result.into())
    }
}
