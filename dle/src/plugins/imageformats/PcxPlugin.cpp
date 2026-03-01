#include "PcxPlugin.h"
#include "PcxHandler.h"

QImageIOPlugin::Capabilities PcxPlugin::capabilities(QIODevice *device, const QByteArray &format) const
{
    if (format == "pcx") {
        return Capabilities(CanRead);
    }
    
    if (!format.isEmpty()) {
        return {};
    }
    
    if (!device->isOpen()) {
        return {};
    }
    
    Capabilities cap;
    if (device->isReadable() && PcxHandler::canRead(device)) {
        cap |= CanRead;
    }
    
    return cap;
}

QImageIOHandler *PcxPlugin::create(QIODevice *device, const QByteArray &format) const
{
    QImageIOHandler *handler = new PcxHandler;
    handler->setDevice(device);
    handler->setFormat(format);
    return handler;
}
