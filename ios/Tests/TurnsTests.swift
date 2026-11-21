import Foundation
import Testing
@testable import ARTEON

/// La file des téléchargements d'images : un nombre limité à la fois, le dernier arrivé
/// d'abord, et les demandeurs annulés qui sortent de la file.
@Suite struct TurnsTests {
    /// Ce qui s'est passé, dans l'ordre.
    actor Journal {
        var entries: [String] = []
        func note(_ entry: String) { entries.append(entry) }
    }

    /// Attend que la file compte `count` demandeurs (ils s'y inscrivent chacun à leur rythme).
    private func waiters(_ turns: Turns, reach count: Int) async {
        for _ in 0..<400 where await turns.waitingCount != count {
            try? await Task.sleep(nanoseconds: 5_000_000)
        }
    }

    @Test func noMoreThanTheLimitAtOnce() async {
        let turns = Turns(limit: 2)
        #expect(await turns.enter())
        #expect(await turns.enter())
        let third = Task { await turns.enter() }
        await waiters(turns, reach: 1)
        // Le troisième attend ; un tour rendu, il passe.
        await turns.leave()
        #expect(await third.value)
        #expect(await turns.waitingCount == 0)
    }

    @Test func theLastArrivedGoesFirst() async {
        let turns = Turns(limit: 1)
        let journal = Journal()
        #expect(await turns.enter())
        var tasks: [Task<Void, Never>] = []
        for name in ["a", "b", "c"] {
            tasks.append(Task {
                if await turns.enter() {
                    await journal.note(name)
                    await turns.leave()
                }
            })
            await waiters(turns, reach: tasks.count)
        }
        await turns.leave()
        for task in tasks { await task.value }
        #expect(await journal.entries == ["c", "b", "a"])
    }

    @Test func aCancelledWaiterLeavesTheQueue() async {
        let turns = Turns(limit: 1)
        #expect(await turns.enter())
        let gone = Task { await turns.enter() }
        let patient = Task { await turns.enter() }
        await waiters(turns, reach: 2)
        gone.cancel()
        #expect(await gone.value == false)
        await waiters(turns, reach: 1)
        // Le tour rendu va à celui qui attend encore, pas à l'annulé.
        await turns.leave()
        #expect(await patient.value)
        // Et la file retrouve sa place : un nouveau venu passe dès que le tour est rendu.
        await turns.leave()
        #expect(await turns.enter())
    }

    @Test func cancelledBeforeWaitingGetsNoTurn() async {
        let turns = Turns(limit: 1)
        #expect(await turns.enter())
        let task = Task { () -> Bool in
            // Déjà annulée quand elle demande son tour.
            while !Task.isCancelled { await Task.yield() }
            return await turns.enter()
        }
        task.cancel()
        #expect(await task.value == false)
        #expect(await turns.waitingCount == 0)
    }
}
