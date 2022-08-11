pub fn generate_negotiate_response(connection_id: &str, connection_token: &str) -> String {
    let mut result = String::new();

    result.push_str("{\"negotiateVersion\":1,\"connectionId\":\"");

    result.push_str(connection_id);

    result.push_str("\",\"connectionToken\":\"");

    result.push_str(connection_token);

    result.push_str("\",\"availableTransports\":[{\"transport\":\"WebSockets\",\"transferFormats\":[\"Text\"]}]}");

    result
}
