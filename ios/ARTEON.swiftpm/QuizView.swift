import SwiftUI

/// Le jeu : une œuvre, quatre artistes. Trouver le bon ; la série s'arrête à la première
/// erreur, et le record est gardé sur l'appareil.
struct QuizView: View {
    let catalogue: Catalogue

    @Environment(CatalogueStore.self) private var store
    @State private var question: QuizQuestion?
    @State private var chosen: String?
    @State private var score = 0
    @State private var lost = false
    @State private var generator = SystemRandomNumberGenerator()

    var body: some View {
        ScrollView {
            VStack(spacing: 20) {
                HStack {
                    Label("\(score)", systemImage: "flame.fill")
                        .font(.title2.bold().monospacedDigit())
                        .foregroundStyle(.orange)
                        .contentTransition(.numericText())
                        .accessibilityLabel("Série : \(score)")
                    Spacer()
                    Label("Record : \(store.quizBest)", systemImage: "trophy.fill")
                        .font(.headline)
                        .foregroundStyle(.secondary)
                }

                if let question, let found = catalogue.work(question.work) {
                    RemoteImage(path: found.1.image, contentMode: .fit, tint: .clear, maxPixels: 1600)
                        .aspectRatio(found.1.aspect, contentMode: .fit)
                        .frame(maxHeight: 420)
                        .padding(8)
                        .background(Color.galleryWall, in: .rect(cornerRadius: 12))
                        .accessibilityLabel("L'œuvre à reconnaître")

                    Text("Qui a fait cette œuvre ?")
                        .font(.title2.bold())

                    if chosen != nil {
                        VStack(spacing: 4) {
                            Text(found.1.title).font(.headline)
                            Text("\(found.1.year) · \(found.0.name)")
                                .font(.subheadline)
                                .foregroundStyle(.secondary)
                        }
                        .multilineTextAlignment(.center)
                        .transition(.opacity)
                    }

                    LazyVGrid(columns: [GridItem(.adaptive(minimum: 240), spacing: 12)], spacing: 12) {
                        ForEach(question.choices, id: \.self) { choice in
                            Button {
                                answer(choice, for: question)
                            } label: {
                                HStack {
                                    Text(choice)
                                        .multilineTextAlignment(.leading)
                                    Spacer()
                                    if let chosen {
                                        if choice == question.answer {
                                            Image(systemName: "checkmark.circle.fill")
                                        } else if choice == chosen {
                                            Image(systemName: "xmark.circle.fill")
                                        }
                                    }
                                }
                                .font(.headline)
                                .padding()
                                .frame(maxWidth: .infinity, minHeight: 56)
                                .background(color(of: choice, in: question), in: .rect(cornerRadius: 14))
                                .foregroundStyle(chosen != nil && (choice == question.answer || choice == chosen) ? Color.white : Color.primary)
                            }
                            .buttonStyle(.plain)
                            .disabled(chosen != nil)
                        }
                    }

                    if chosen != nil {
                        HStack(spacing: 12) {
                            NavigationLink(value: Route.work(question.work)) {
                                Label("Voir la notice", systemImage: "book")
                            }
                            .buttonStyle(.glass)
                            Button(lost ? "Rejouer" : "Suivante", systemImage: lost ? "arrow.counterclockwise" : "arrow.right") {
                                next()
                            }
                            .buttonStyle(.glassProminent)
                        }
                    }
                } else {
                    ProgressView()
                }
            }
            .padding()
            .frame(maxWidth: 760)
            .frame(maxWidth: .infinity)
        }
        .background(Color(uiColor: .systemGroupedBackground))
        .navigationTitle("Qui l'a fait ?")
        .navigationBarTitleDisplayMode(.inline)
        .sensoryFeedback(trigger: chosen) { _, new in
            guard let new, let question else { return nil }
            return new == question.answer ? .success : .error
        }
        .task { if question == nil { next() } }
    }

    private func color(of choice: String, in question: QuizQuestion) -> Color {
        guard chosen != nil else { return Color.card }
        if choice == question.answer { return .green }
        if choice == chosen { return .red }
        return Color.card
    }

    private func answer(_ choice: String, for question: QuizQuestion) {
        withAnimation(.snappy) {
            chosen = choice
            if choice == question.answer {
                score += 1
                if score > store.quizBest { store.quizBest = score }
            } else {
                lost = true
            }
        }
    }

    private func next() {
        guard let index = store.index else { return }
        if lost {
            score = 0
            lost = false
        }
        withAnimation {
            chosen = nil
            question = Quiz.question(from: catalogue, index: index, using: &generator)
        }
    }
}
