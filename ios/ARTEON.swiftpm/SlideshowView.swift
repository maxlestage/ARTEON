import SwiftUI

/// Le diaporama : les œuvres en grand, l'une après l'autre, sur fond noir ; balayer pour
/// passer, toucher pour montrer ou cacher les commandes. L'écran reste allumé.
struct SlideshowView: View {
    let title: String
    let items: [(Exhibition, Work)]

    @Environment(\.dismiss) private var dismiss
    @State private var index: Int? = 0
    @State private var playing = true
    @State private var chrome = true
    @State private var seconds = 8.0

    private var current: (Exhibition, Work)? {
        guard let index, items.indices.contains(index) else { return nil }
        return items[index]
    }

    var body: some View {
        ZStack {
            Color.black.ignoresSafeArea()

            ScrollView(.horizontal) {
                LazyHStack(spacing: 0) {
                    ForEach(items.indices, id: \.self) { i in
                        RemoteImage(path: items[i].1.image, contentMode: .fit, tint: .clear, maxPixels: 2400)
                            .padding(chrome ? 60 : 16)
                            .containerRelativeFrame([.horizontal, .vertical])
                            .id(i)
                    }
                }
                .scrollTargetLayout()
            }
            .scrollTargetBehavior(.paging)
            .scrollPosition(id: $index)
            .scrollIndicators(.hidden)
            .ignoresSafeArea()
            .onTapGesture { withAnimation { chrome.toggle() } }

            if chrome {
                VStack {
                    HStack {
                        Text(title)
                            .font(.headline)
                        Spacer()
                        Button("Fermer", systemImage: "xmark") { dismiss() }
                            .labelStyle(.iconOnly)
                            .buttonStyle(.glass)
                    }
                    Spacer()
                    if let shown = current {
                        let ex = shown.0
                        let work = shown.1
                        VStack(alignment: .leading, spacing: 10) {
                            Text(work.title)
                                .font(.title2.bold())
                            Text([work.byline, ex.name].filter { !$0.isEmpty }.joined(separator: " · "))
                                .font(.subheadline)
                                .opacity(0.8)
                            HStack(spacing: 14) {
                                Button(playing ? "Pause" : "Lecture", systemImage: playing ? "pause.fill" : "play.fill") {
                                    playing.toggle()
                                }
                                .buttonStyle(.glassProminent)
                                Menu {
                                    Picker("Durée", selection: $seconds) {
                                        Text("4 secondes").tag(4.0)
                                        Text("8 secondes").tag(8.0)
                                        Text("15 secondes").tag(15.0)
                                        Text("30 secondes").tag(30.0)
                                    }
                                } label: {
                                    Label("\(Int(seconds)) s", systemImage: "timer")
                                }
                                .buttonStyle(.glass)
                                Spacer()
                                Text("\((index ?? 0) + 1) / \(items.count)")
                                    .font(.subheadline.monospacedDigit())
                                    .opacity(0.8)
                            }
                        }
                        .padding()
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .glassEffect(in: .rect(cornerRadius: 22))
                    }
                }
                .padding()
                .foregroundStyle(.white)
                .transition(.opacity)
            }
        }
        .preferredColorScheme(.dark)
        .statusBarHidden(!chrome)
        .persistentSystemOverlays(chrome ? .automatic : .hidden)
        .onAppear { UIApplication.shared.isIdleTimerDisabled = true }
        .onDisappear { UIApplication.shared.isIdleTimerDisabled = false }
        .task(id: "\(playing)-\(seconds)-\(index ?? 0)") {
            guard playing, items.count > 1 else { return }
            try? await Task.sleep(for: .seconds(seconds))
            guard !Task.isCancelled else { return }
            withAnimation(.easeInOut(duration: 0.8)) {
                index = ((index ?? 0) + 1) % items.count
            }
        }
    }
}
