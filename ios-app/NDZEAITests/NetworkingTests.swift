import XCTest
@testable import NDZEAI

final class NetworkingTests: XCTestCase {
    private func config(_ format: APIFormat, template: String = APIFormat.defaultTemplate) -> EndpointConfig {
        EndpointConfig(
            baseURL: "https://api.example.com/", path: format.defaultPath, format: format, model: "m1",
            apiKey: "tc_live_secret", authHeader: format.defaultAuthHeader, authPrefix: format.defaultAuthPrefix,
            bodyTemplate: template, responsePath: format.defaultResponsePath, systemPrompt: "sys", streaming: true
        )
    }

    func testKeyMaskMatchesDesign() {
        XCTAssertEqual(KeyMask.mask("tc_live_9f8e7d6c5b4a3f2e1b33"), "tc_live_****************1b33")
        XCTAssertEqual(KeyMask.mask("sk-abcdefghijklmnop1234"), "sk-****************1234")
        XCTAssertEqual(KeyMask.mask("short"), "*****")
    }

    func testJSONPath() {
        let data = Data(#"{"choices":[{"message":{"content":"hi"}}]}"#.utf8)
        XCTAssertEqual(JSONPath.string(at: "choices.0.message.content", in: data), "hi")
        XCTAssertNil(JSONPath.string(at: "choices.1.message.content", in: data))
        XCTAssertEqual(JSONPath.string(at: "", in: Data("plain text".utf8)), "plain text")
    }

    func testTemplateEscapesUserContent() throws {
        let turns = [ChatTurn(role: "user", content: #"say "{{model}}" \ now"#)]
        let data = try BodyTemplate.render(#"{"q": {{prompt}}, "m": {{model}}, "h": {{messages}}}"#, turns: turns, model: "m1", system: "")
        let object = try XCTUnwrap(JSONSerialization.jsonObject(with: data) as? [String: Any])
        XCTAssertEqual(object["q"] as? String, #"say "{{model}}" \ now"#)
        XCTAssertEqual(object["m"] as? String, "m1")
        XCTAssertEqual((object["h"] as? [[String: String]])?.first?["role"], "user")
    }

    func testTemplateRejectsInvalidJSON() {
        XCTAssertThrowsError(try BodyTemplate.render("{\"q\": {{prompt}", turns: [], model: "", system: ""))
    }

    func testSSEParsing() {
        XCTAssertEqual(SSEParser.parse(line: #"data: {"choices":[{"delta":{"content":"Hel"}}]}"#, format: .openAI), .delta("Hel"))
        XCTAssertEqual(SSEParser.parse(line: "data: [DONE]", format: .openAI), .done)
        XCTAssertEqual(SSEParser.parse(line: #"data: {"type":"content_block_delta","delta":{"type":"text_delta","text":"lo"}}"#, format: .anthropic), .delta("lo"))
        XCTAssertEqual(SSEParser.parse(line: #"data: {"type":"error","error":{"message":"overloaded"}}"#, format: .anthropic), .failure("overloaded"))
        XCTAssertNil(SSEParser.parse(line: "event: ping", format: .anthropic))
    }

    func testOpenAIRequest() throws {
        let request = try AIClient().makeRequest([ChatTurn(role: "user", content: "hi")], config: config(.openAI))
        XCTAssertEqual(request.url?.absoluteString, "https://api.example.com/v1/chat/completions")
        XCTAssertEqual(request.value(forHTTPHeaderField: "Authorization"), "Bearer tc_live_secret")
        let body = try XCTUnwrap(JSONSerialization.jsonObject(with: XCTUnwrap(request.httpBody)) as? [String: Any])
        let messages = try XCTUnwrap(body["messages"] as? [[String: String]])
        XCTAssertEqual(messages.map { $0["role"] }, ["system", "user"])
        XCTAssertEqual(body["stream"] as? Bool, true)
    }

    func testAnthropicRequest() throws {
        let request = try AIClient().makeRequest([ChatTurn(role: "user", content: "hi")], config: config(.anthropic))
        XCTAssertEqual(request.value(forHTTPHeaderField: "x-api-key"), "tc_live_secret")
        XCTAssertEqual(request.value(forHTTPHeaderField: "anthropic-version"), "2023-06-01")
        let body = try XCTUnwrap(JSONSerialization.jsonObject(with: XCTUnwrap(request.httpBody)) as? [String: Any])
        XCTAssertEqual(body["system"] as? String, "sys")
        XCTAssertEqual(body["model"] as? String, "m1")
    }

    func testCustomFormatNeverStreams() {
        XCTAssertFalse(config(.custom).usesStreaming)
        var bad = config(.openAI)
        bad.baseURL = "ftp://x"
        XCTAssertNil(bad.url)
    }
}
