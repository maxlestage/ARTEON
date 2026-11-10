// swift-tools-version: 6.0

// ARTEON pour iPad et iPhone, en Swift et SwiftUI.
//
// Ce dossier est un « App Playground » : sur l'iPad, Swift Playgrounds l'ouvre, le
// compile et lance l'application, sans Mac (voir ios/README.md). Le même code est
// compilé par Xcode sur un Mac de GitHub (ios/project.yml) pour TestFlight.

import PackageDescription
import AppleProductTypes

let package = Package(
    name: "ARTEON",
    platforms: [
        .iOS("26.0")
    ],
    products: [
        .iOSApplication(
            name: "ARTEON",
            targets: ["AppModule"],
            bundleIdentifier: "com.maxlestage.arteon",
            teamIdentifier: "",
            displayVersion: "1.0",
            bundleVersion: "1",
            appIcon: .asset("AppIcon"),
            accentColor: .asset("AccentColor"),
            supportedDeviceFamilies: [
                .pad,
                .phone
            ],
            supportedInterfaceOrientations: [
                .portrait,
                .landscapeRight,
                .landscapeLeft,
                .portraitUpsideDown(.when(deviceFamilies: [.pad]))
            ],
            appCategory: .education
        )
    ],
    targets: [
        .executableTarget(
            name: "AppModule",
            path: "."
        )
    ],
    swiftLanguageModes: [.v6]
)
