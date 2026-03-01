#pragma once

#include <QImageIOPlugin>

/**
 * @brief Qt plugin for loading PCX (PC Paintbrush) image format
 * 
 * This plugin allows Qt applications to load PCX images using
 * standard Qt APIs like QImage::load() and QPixmap::load().
 * 
 * The plugin will be automatically registered with Qt's image I/O system.
 */
class PcxPlugin : public QImageIOPlugin
{
    Q_OBJECT
    Q_PLUGIN_METADATA(IID "org.qt-project.Qt.QImageIOHandlerFactoryInterface" FILE "pcx.json")
    
public:
    Capabilities capabilities(QIODevice *device, const QByteArray &format) const override;
    QImageIOHandler *create(QIODevice *device, const QByteArray &format = QByteArray()) const override;
};
