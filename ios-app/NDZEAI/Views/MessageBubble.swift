import SwiftUI
import UIKit

struct MessageBubble: View {
    let message: ChatMessage

    private var isUser: Bool { message.role == .user }

    var body: some View {
        HStack {
            if isUser { Spacer(minLength: 48) }
            content
                .padding(.horizontal, 14)
                .padding(.vertical, 10)
                .foregroundStyle(isUser ? Color.black : Color.primary)
                .background(background, in: RoundedRectangle(cornerRadius: 20, style: .continuous))
                .contextMenu {
                    Button("Копировать", systemImage: "doc.on.doc") {
                        UIPasteboard.general.string = message.content
                    }
                }
            if !isUser { Spacer(minLength: 48) }
        }
    }

    @ViewBuilder
    private var content: some View {
        if message.content.isEmpty {
            Image(systemName: "ellipsis")
                .font(.title3)
                .symbolEffect(.variableColor.iterative, options: .repeating)
                .accessibilityLabel("Печатает")
        } else if message.isError {
            Label(message.content, systemImage: "exclamationmark.triangle.fill")
                .font(.callout)
        } else {
            Text(markdown)
                .textSelection(.enabled)
        }
    }

    private var background: AnyShapeStyle {
        if message.isError { return AnyShapeStyle(Color.red.opacity(0.18)) }
        if isUser {
            return AnyShapeStyle(LinearGradient(colors: [.mint, .cyan], startPoint: .topLeading, endPoint: .bottomTrailing))
        }
        return AnyShapeStyle(Color(.secondarySystemBackground))
    }

    private var markdown: AttributedString {
        let options = AttributedString.MarkdownParsingOptions(interpretedSyntax: .inlineOnlyPreservingWhitespace)
        return (try? AttributedString(markdown: message.content, options: options)) ?? AttributedString(message.content)
    }
}
