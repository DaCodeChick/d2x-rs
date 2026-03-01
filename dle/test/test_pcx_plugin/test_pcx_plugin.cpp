#include <QCoreApplication>
#include <QImage>
#include <QDebug>
#include <QPluginLoader>
#include <QDir>

int main(int argc, char *argv[])
{
    QCoreApplication app(argc, argv);
    
    if (argc < 2) {
        qWarning() << "Usage:" << argv[0] << "<pcx_file>";
        return 1;
    }
    
    // Add plugin path
    QString pluginPath = QCoreApplication::applicationDirPath() + "/../../plugins/imageformats";
    QCoreApplication::addLibraryPath(pluginPath);
    
    qDebug() << "Plugin paths:" << QCoreApplication::libraryPaths();
    
    // Try to load the PCX file
    QString pcxFile = argv[1];
    QImage image;
    
    qDebug() << "Loading PCX file:" << pcxFile;
    
    if (image.load(pcxFile)) {
        qDebug() << "✓ Successfully loaded PCX image";
        qDebug() << "  Size:" << image.width() << "x" << image.height();
        qDebug() << "  Format:" << image.format();
        qDebug() << "  Depth:" << image.depth() << "bits per pixel";
        
        if (image.format() == QImage::Format_Indexed8) {
            qDebug() << "  Color table size:" << image.colorCount();
        }
        
        // Save as PNG to verify it worked
        QString pngFile = pcxFile.left(pcxFile.lastIndexOf('.')) + ".png";
        if (image.save(pngFile)) {
            qDebug() << "✓ Saved as PNG:" << pngFile;
        }
        
        return 0;
    } else {
        qWarning() << "✗ Failed to load PCX file";
        return 1;
    }
}
