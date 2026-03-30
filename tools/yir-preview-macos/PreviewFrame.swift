import AppKit
import Foundation

// This previewer is a macOS-specific adapter over the current CPU-hosted
// preview path. It consumes a derived UI plan and framebuffer artifact; it is
// not part of YIR core semantics.

struct PpmImage {
    let width: Int
    let height: Int
    let pixels: Data
}

struct UiInput {
    let channel: String
    let defaultValue: Int
}

struct UiPlan {
    let title: String
    let width: Int
    let height: Int
    let inputs: [UiInput]
}

enum PpmError: Error, CustomStringConvertible {
    case invalidHeader
    case invalidToken
    case unsupportedMaxValue(Int)
    case truncatedPixelData

    var description: String {
        switch self {
        case .invalidHeader:
            return "invalid PPM header"
        case .invalidToken:
            return "invalid PPM token"
        case .unsupportedMaxValue(let value):
            return "unsupported PPM max value \(value)"
        case .truncatedPixelData:
            return "truncated PPM pixel data"
        }
    }
}

func parseUiPlan(at path: String) throws -> UiPlan {
    let text = try String(contentsOfFile: path, encoding: .utf8)
    var title = "Nuis YIR UI Preview"
    var width = 640
    var height = 480
    var inputs: [UiInput] = []

    for rawLine in text.split(whereSeparator: \.isNewline) {
        let line = rawLine.trimmingCharacters(in: .whitespacesAndNewlines)
        if line.isEmpty { continue }

        if let value = line.split(separator: "=", maxSplits: 1).dropFirst().first, line.hasPrefix("window.title=") {
            title = String(value)
        } else if let value = line.split(separator: "=", maxSplits: 1).dropFirst().first, line.hasPrefix("window.width=") {
            width = Int(value) ?? width
        } else if let value = line.split(separator: "=", maxSplits: 1).dropFirst().first, line.hasPrefix("window.height=") {
            height = Int(value) ?? height
        } else if line.hasPrefix("input=") {
            let payload = line.dropFirst("input=".count)
            let parts = payload.split(separator: ",", maxSplits: 1).map(String.init)
            if parts.count == 2, let defaultValue = Int(parts[1]) {
                inputs.append(UiInput(channel: parts[0], defaultValue: defaultValue))
            }
        }
    }

    return UiPlan(title: title, width: width, height: height, inputs: inputs)
}

func parsePpm(at path: String) throws -> PpmImage {
    let data = try Data(contentsOf: URL(fileURLWithPath: path))
    var index = data.startIndex

    func skipWhitespaceAndComments() {
        while index < data.endIndex {
            let byte = data[index]
            if byte == 35 {
                while index < data.endIndex && data[index] != 10 {
                    index = data.index(after: index)
                }
            } else if byte == 9 || byte == 10 || byte == 13 || byte == 32 {
                index = data.index(after: index)
            } else {
                break
            }
        }
    }

    func readToken() throws -> String {
        skipWhitespaceAndComments()
        let start = index
        while index < data.endIndex {
            let byte = data[index]
            if byte == 9 || byte == 10 || byte == 13 || byte == 32 || byte == 35 {
                break
            }
            index = data.index(after: index)
        }

        guard start < index, let token = String(data: data[start..<index], encoding: .utf8) else {
            throw PpmError.invalidToken
        }
        return token
    }

    let magic = try readToken()
    guard magic == "P6" else {
        throw PpmError.invalidHeader
    }

    guard let width = Int(try readToken()), let height = Int(try readToken()) else {
        throw PpmError.invalidToken
    }

    guard let maxValue = Int(try readToken()) else {
        throw PpmError.invalidToken
    }

    guard maxValue == 255 else {
        throw PpmError.unsupportedMaxValue(maxValue)
    }

    if index < data.endIndex && (data[index] == 10 || data[index] == 13 || data[index] == 32) {
        index = data.index(after: index)
    }

    let expected = width * height * 3
    let remaining = data.distance(from: index, to: data.endIndex)
    guard remaining >= expected else {
        throw PpmError.truncatedPixelData
    }

    let pixels = data[index..<data.index(index, offsetBy: expected)]
    return PpmImage(width: width, height: height, pixels: Data(pixels))
}

func makeImage(from ppm: PpmImage) -> NSImage? {
    guard let bitmap = NSBitmapImageRep(
        bitmapDataPlanes: nil,
        pixelsWide: ppm.width,
        pixelsHigh: ppm.height,
        bitsPerSample: 8,
        samplesPerPixel: 3,
        hasAlpha: false,
        isPlanar: false,
        colorSpaceName: .deviceRGB,
        bytesPerRow: ppm.width * 3,
        bitsPerPixel: 24
    ) else {
        return nil
    }

    ppm.pixels.withUnsafeBytes { bytes in
        guard let src = bytes.baseAddress, let dst = bitmap.bitmapData else { return }
        memcpy(dst, src, ppm.pixels.count)
    }

    let image = NSImage(size: NSSize(width: ppm.width, height: ppm.height))
    image.addRepresentation(bitmap)
    return image
}

final class AppDelegate: NSObject, NSApplicationDelegate {
    private let planPath: String
    private let modulePath: String
    private let imagePath: String
    private let scale: String
    private let rootDir: String
    private let exportBinaryPath: String
    private var window: NSWindow?
    private var imageView: NSImageView?
    private var sliders: [String: NSSlider] = [:]
    private var lastRenderedInputs: [String: Int] = [:]
    private var rerenderWorkItem: DispatchWorkItem?

    init(
        planPath: String,
        modulePath: String,
        imagePath: String,
        scale: String,
        rootDir: String,
        exportBinaryPath: String
    ) {
        self.planPath = planPath
        self.modulePath = modulePath
        self.imagePath = imagePath
        self.scale = scale
        self.rootDir = rootDir
        self.exportBinaryPath = exportBinaryPath
    }

    func applicationDidFinishLaunching(_ notification: Notification) {
        do {
            let plan = try parseUiPlan(at: planPath)
            let ppm = try parsePpm(at: imagePath)
            guard let image = makeImage(from: ppm) else {
                fputs("failed to create image from framebuffer\n", stderr)
                NSApp.terminate(nil)
                return
            }

            let previewScale: CGFloat = 2.0
            let previewRect = NSRect(
                x: 0,
                y: 80,
                width: CGFloat(ppm.width) * previewScale,
                height: CGFloat(ppm.height) * previewScale
            )
            let windowRect = NSRect(
                x: 0,
                y: 0,
                width: max(CGFloat(plan.width), previewRect.width),
                height: max(CGFloat(plan.height), previewRect.height + 90)
            )
            let window = NSWindow(
                contentRect: windowRect,
                styleMask: [.titled, .closable, .miniaturizable, .resizable],
                backing: .buffered,
                defer: false
            )
            window.center()
            window.title = plan.title
            window.isReleasedWhenClosed = false

            let imageView = NSImageView(frame: previewRect)
            imageView.image = image
            imageView.imageScaling = .scaleAxesIndependently
            imageView.autoresizingMask = [.width, .height]

            let content = NSView(frame: windowRect)
            content.addSubview(imageView)
            self.imageView = imageView

            var sliderY = CGFloat(48)
            for input in plan.inputs {
                let label = NSTextField(labelWithString: input.channel)
                label.frame = NSRect(x: 16, y: sliderY + 2, width: 120, height: 20)
                label.textColor = .labelColor
                content.addSubview(label)

                let slider = NSSlider(value: Double(input.defaultValue), minValue: 0, maxValue: 255, target: self, action: #selector(sliderChanged(_:)))
                slider.identifier = NSUserInterfaceItemIdentifier(rawValue: input.channel)
                slider.isContinuous = false
                slider.frame = NSRect(x: 140, y: sliderY, width: windowRect.width - 200, height: 24)
                slider.autoresizingMask = [.width]
                content.addSubview(slider)
                sliders[input.channel] = slider
                lastRenderedInputs[input.channel] = input.defaultValue

                let valueLabel = NSTextField(labelWithString: "\(input.defaultValue)")
                valueLabel.frame = NSRect(x: windowRect.width - 52, y: sliderY + 2, width: 40, height: 20)
                valueLabel.identifier = NSUserInterfaceItemIdentifier(rawValue: "\(input.channel)__value")
                valueLabel.alignment = .right
                valueLabel.autoresizingMask = [.minXMargin]
                content.addSubview(valueLabel)

                sliderY += 28
            }

            window.contentView = content
            window.makeKeyAndOrderFront(nil)
            self.window = window
            NSApp.activate(ignoringOtherApps: true)
        } catch {
            fputs("preview failed: \(error)\n", stderr)
            NSApp.terminate(nil)
        }
    }

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        true
    }

    @objc
    private func sliderChanged(_ sender: NSSlider) {
        guard let channel = sender.identifier?.rawValue else { return }
        if let label = window?.contentView?.subviews.first(where: { $0.identifier?.rawValue == "\(channel)__value" }) as? NSTextField {
            label.stringValue = "\(Int(sender.integerValue))"
        }
        scheduleRenderFromCurrentControls()
    }

    private func scheduleRenderFromCurrentControls() {
        rerenderWorkItem?.cancel()
        let workItem = DispatchWorkItem { [weak self] in
            self?.renderFromCurrentControls()
        }
        rerenderWorkItem = workItem
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.05, execute: workItem)
    }

    private func renderFromCurrentControls() {
        var currentInputs: [String: Int] = [:]
        for (channel, slider) in sliders {
            currentInputs[channel] = slider.integerValue
        }
        if currentInputs == lastRenderedInputs {
            return
        }

        let process = Process()
        process.currentDirectoryURL = URL(fileURLWithPath: rootDir)
        process.executableURL = URL(fileURLWithPath: exportBinaryPath)
        process.arguments = [modulePath, imagePath, scale]

        var environment = ProcessInfo.processInfo.environment
        for (channel, value) in currentInputs {
            environment["NUIS_UI_\(normalizeChannel(channel))"] = "\(value)"
        }
        process.environment = environment

        do {
            try process.run()
            process.waitUntilExit()
            if process.terminationStatus == 0 {
                lastRenderedInputs = currentInputs
                reloadPreviewImage()
            }
        } catch {
            fputs("failed to rerender YIR frame: \(error)\n", stderr)
        }
    }

    private func reloadPreviewImage() {
        do {
            let ppm = try parsePpm(at: imagePath)
            if let image = makeImage(from: ppm) {
                imageView?.image = image
            }
        } catch {
            fputs("failed to reload preview image: \(error)\n", stderr)
        }
    }
}

func normalizeChannel(_ channel: String) -> String {
    channel
        .map { ch in
            if ch.isLetter || ch.isNumber {
                return String(ch).uppercased()
            }
            return "_"
        }
        .joined()
}

guard CommandLine.arguments.count >= 7 else {
    fputs("usage: PreviewFrame <ui.plan> <module.yir> <frame.ppm> <scale> <root-dir> <yir-export-frame>\n", stderr)
    exit(1)
}

let app = NSApplication.shared
let delegate = AppDelegate(
    planPath: CommandLine.arguments[1],
    modulePath: CommandLine.arguments[2],
    imagePath: CommandLine.arguments[3],
    scale: CommandLine.arguments[4],
    rootDir: CommandLine.arguments[5],
    exportBinaryPath: CommandLine.arguments[6]
)
app.setActivationPolicy(.regular)
app.delegate = delegate
app.run()
