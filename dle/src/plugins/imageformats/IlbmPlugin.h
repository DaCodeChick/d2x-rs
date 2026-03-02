#pragma once

#include <QImageIOPlugin>

/**
 * @brief Qt plugin for loading IFF ILBM/PBM image format
 * 
 * This plugin allows Qt applications to load IFF/ILBM/PBM images using
 * standard Qt APIs like QImage::load() and QPixmap::load().
 * 
 * IFF (Interchange File Format) files are used in Descent for briefing
 * screens and typically have a `.bbm` extension.
 * 
 * The plugin will be automatically registered with Qt's image I/O system.
 */
class IlbmPlugin : public QImageIOPlugin
{
    Q_OBJECT
    Q_PLUGIN_METADATA(IID "org.qt-project.Qt.QImageIOHandlerFactoryInterface" FILE "ilbm.json")
    
public:
    Capabilities capabilities(QIODevice *device, const QByteArray &format) const override;
    QImageIOHandler *create(QIODevice *device, const QByteArray &format = QByteArray()) const override;
};
