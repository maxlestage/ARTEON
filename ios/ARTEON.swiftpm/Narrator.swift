// L'audioguide : la voix française de l'appareil lit le cartel et la notice d'une œuvre,
// ou l'introduction d'une exposition. Sans Internet, même en mode silencieux.

import AVFoundation
import Observation
import SwiftUI

@MainActor
@Observable
final class Narrator {
    /// Ce qui est lu en ce moment (la clé d'une œuvre, le nom d'une exposition).
    private(set) var current: String?
    private(set) var paused = false

    /// La vitesse de lecture, de 0,3 (lent) à 0,6 (rapide).
    var rate: Double {
        didSet { UserDefaults.standard.set(rate, forKey: "debitVoix") }
    }

    @ObservationIgnored private let synthesizer = AVSpeechSynthesizer()
    @ObservationIgnored private var delegate: SpeechDelegate?
    /// La phrase en cours : la fin d'une lecture interrompue n'efface pas la suivante.
    @ObservationIgnored private var utterance: ObjectIdentifier?

    init() {
        let saved = UserDefaults.standard.double(forKey: "debitVoix")
        rate = saved > 0 ? saved : 0.48
        let delegate = SpeechDelegate { [weak self] finished in
            Task { @MainActor in
                guard let self, self.utterance == finished else { return }
                self.utterance = nil
                self.current = nil
                self.paused = false
            }
        }
        self.delegate = delegate
        synthesizer.delegate = delegate
    }

    func isReading(_ id: String) -> Bool { current == id }

    /// Lit `text` ; relancé sur ce qui est déjà lu, met en pause ou reprend.
    func toggle(_ id: String, text: String) {
        if current == id {
            if paused {
                synthesizer.continueSpeaking()
                paused = false
            } else {
                synthesizer.pauseSpeaking(at: .word)
                paused = true
            }
            return
        }
        stop()
        try? AVAudioSession.sharedInstance().setCategory(.playback, mode: .spokenAudio)
        try? AVAudioSession.sharedInstance().setActive(true)
        let utterance = AVSpeechUtterance(string: text)
        utterance.voice = Self.voice
        utterance.rate = Float(rate)
        utterance.postUtteranceDelay = 0.2
        current = id
        paused = false
        self.utterance = ObjectIdentifier(utterance)
        synthesizer.speak(utterance)
    }

    func stop() {
        if synthesizer.isSpeaking { synthesizer.stopSpeaking(at: .immediate) }
        utterance = nil
        current = nil
        paused = false
    }

    /// La plus belle voix française installée.
    private static var voice: AVSpeechSynthesisVoice? {
        let french = AVSpeechSynthesisVoice.speechVoices().filter { $0.language.hasPrefix("fr") }
        return french.max { a, b in
            (a.language == "fr-FR" ? 10 : 0) + a.quality.rawValue < (b.language == "fr-FR" ? 10 : 0) + b.quality.rawValue
        } ?? AVSpeechSynthesisVoice(language: "fr-FR")
    }
}

/// Prévient le narrateur quand la lecture s'achève.
private final class SpeechDelegate: NSObject, AVSpeechSynthesizerDelegate, Sendable {
    let finished: @Sendable (ObjectIdentifier) -> Void

    init(finished: @escaping @Sendable (ObjectIdentifier) -> Void) {
        self.finished = finished
    }

    func speechSynthesizer(_ synthesizer: AVSpeechSynthesizer, didFinish utterance: AVSpeechUtterance) {
        finished(ObjectIdentifier(utterance))
    }

    func speechSynthesizer(_ synthesizer: AVSpeechSynthesizer, didCancel utterance: AVSpeechUtterance) {
        finished(ObjectIdentifier(utterance))
    }
}

/// Le bouton de l'audioguide : écouter, pause, reprendre.
struct ListenButton: View {
    let id: String
    let text: () -> String
    var label: String = "Écouter"

    @Environment(Narrator.self) private var narrator

    var body: some View {
        let reading = narrator.isReading(id)
        Button {
            narrator.toggle(id, text: text())
        } label: {
            Label(
                reading ? (narrator.paused ? "Reprendre" : "Pause") : label,
                systemImage: reading ? (narrator.paused ? "play.fill" : "pause.fill") : "speaker.wave.2.fill"
            )
        }
        .symbolEffect(.variableColor.iterative, isActive: reading && !narrator.paused)
        .accessibilityHint("L'audioguide lit la notice à voix haute")
    }
}
