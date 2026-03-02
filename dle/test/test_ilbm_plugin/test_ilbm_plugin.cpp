#include <QCoreApplication>
#include <QImage>
#include <QDebug>
#include <QPluginLoader>
#include <QDir>
#include <print>

int main(int argc, char *argv[])
{
    QCoreApplication app(argc, argv);
    
    if (argc < 2) {
        std::println(stderr, "Usage: {} <ilbm_file>", argv[0]);
        return 1;
    }
    
    // Add plugin path
    QString pluginPath = QCoreApplication::applicationDirPath() + "/../../plugins/imageformats";
    QCoreApplication::addLibraryPath(pluginPath);
    
    std::println("Plugin paths: {}", QCoreApplication::libraryPaths().join(", ").toStdString());
    
    // Try to load the IFF/ILBM file
    QString ilbmFile = argv[1];
    QImage image;
    
    std::println("Loading IFF/ILBM file: {}", ilbmFile.toStdString());
    
    if (image.load(ilbmFile)) {
        std::println("✓ Successfully loaded IFF/ILBM image");
        std::println("  Size: {}x{}", image.width(), image.height());
        std::println("  Format: {}", static_cast<int>(image.format()));
        std::println("  Depth: {} bits per pixel", image.depth());
        
        if (image.format() == QImage::Format_Indexed8) {
            std::println("  Color table size: {}", image.colorCount());
        }
        
        // Save as PNG to verify it worked
        QString pngFile = ilbmFile.left(ilbmFile.lastIndexOf('.')) + ".png";
        if (image.save(pngFile)) {
            std::println("✓ Saved as PNG: {}", pngFile.toStdString());
        }
        
        return 0;
    } else {
        std::println(stderr, "✗ Failed to load IFF/ILBM file");
        return 1;
    }
}
