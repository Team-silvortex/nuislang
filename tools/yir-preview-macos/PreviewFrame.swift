import AppKit
import Foundation

struct PpmImage {
    let width: Int
    let height: Int
    let pixels: Data
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
    private let imagePath: String
    private var window: NSWindow?

    init(imagePath: String) {
        self.imagePath = imagePath
    }

    func applicationDidFinishLaunching(_ notification: Notification) {
        do {
            let ppm = try parsePpm(at: imagePath)
            guard let image = makeImage(from: ppm) else {
                fputs("failed to create image from framebuffer\n", stderr)
                NSApp.terminate(nil)
                return
            }

            let scale: CGFloat = 4.0
            let rect = NSRect(x: 0, y: 0, width: CGFloat(ppm.width) * scale, height: CGFloat(ppm.height) * scale)
            let window = NSWindow(
                contentRect: rect,
                styleMask: [.titled, .closable, .miniaturizable, .resizable],
                backing: .buffered,
                defer: false
            )
            window.center()
            window.title = "Nuis YIR Frame Preview"
            window.isReleasedWhenClosed = false

            let imageView = NSImageView(frame: rect)
            imageView.image = image
            imageView.imageScaling = .scaleAxesIndependently
            imageView.autoresizingMask = [.width, .height]

            let content = NSView(frame: rect)
            content.addSubview(imageView)
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
}

guard CommandLine.arguments.count >= 2 else {
    fputs("usage: PreviewFrame <frame.ppm>\n", stderr)
    exit(1)
}

let app = NSApplication.shared
let delegate = AppDelegate(imagePath: CommandLine.arguments[1])
app.setActivationPolicy(.regular)
app.delegate = delegate
app.run()
