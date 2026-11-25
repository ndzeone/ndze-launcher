import SwiftData
import SwiftUI

struct ConnectionsView: View {
    @Environment(\.modelContext) private var context
    @Environment(\.dismiss) private var dismiss
    @Query(sort: \Connection.createdAt) private var connections: [Connection]

    var body: some View {
        List {
            ForEach(connections) { connection in
                NavigationLink {
                    ConnectionEditor(connection: connection)
                } label: {
                    ConnectionRow(connection: connection)
                }
            }
            .onDelete { offsets in
                for offset in offsets {
                    let connection = connections[offset]
                    Keychain.delete(account: connection.keychainAccount)
                    context.delete(connection)
                }
            }
        }
        .overlay {
            if connections.isEmpty {
                ContentUnavailableView(
                    "Нет ключей",
                    systemImage: "key.horizontal",
                    description: Text("Нажмите «+», чтобы добавить сервер и API-ключ.")
                )
            }
        }
        .navigationTitle("API-ключи")
        .toolbar {
            ToolbarItem(placement: .cancellationAction) {
                Button("Готово") { dismiss() }
            }
            ToolbarItem(placement: .primaryAction) {
                NavigationLink {
                    ConnectionEditor(connection: nil)
                } label: {
                    Label("Добавить", systemImage: "plus")
                }
            }
        }
    }
}

private struct ConnectionRow: View {
    let connection: Connection

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(connection.name)
                .font(.headline)
            Text(connection.maskedKey.isEmpty ? "без ключа" : connection.maskedKey)
                .font(.system(.callout, design: .monospaced))
                .lineLimit(1)
                .padding(.horizontal, 12)
                .padding(.vertical, 10)
                .frame(maxWidth: .infinity, alignment: .leading)
                .background(
                    RoundedRectangle(cornerRadius: 12, style: .continuous)
                        .strokeBorder(Color(.separator))
                )
            Text("\(URL(string: connection.baseURL)?.host() ?? connection.baseURL) · \(connection.format.title)")
                .font(.caption)
                .foregroundStyle(.secondary)
        }
        .padding(.vertical, 4)
    }
}
