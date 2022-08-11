pub fn generate_negotiate_response(
    negotiate_version: usize,
    connection_id: &str,
    connection_token: &Option<String>,
) -> String {
    let mut result = String::new();

    result.push_str("{\"negotiateVersion\":");

    result.push_str(negotiate_version.to_string().as_str());

    result.push_str(",\"connectionId\":\"");

    result.push_str(connection_id);
    result.push_str("\"");

    if let Some(connection_token) = connection_token {
        result.push_str(",\"connectionToken\":\"");
        result.push_str(connection_token.as_str());
        result.push_str("\"")
    }

    result.push_str(
        ",\"availableTransports\":[{\"transport\":\"WebSockets\",\"transferFormats\":[\"Text\", \"Binary\"]},{\"transport\":\"ServerSentEvents\",\"transferFormats\":[\"Text\"]},{\"transport\":\"LongPolling\",\"transferFormats\":[\"Text\",\"Binary\"]}]}",
    );

    result
}

pub fn get_ping_payload() -> &'static str {
    "{\"type\":6}"
}
