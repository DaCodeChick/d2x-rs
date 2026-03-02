#include "IlbmPlugin.h"
#include "IlbmHandler.h"

QImageIOPlugin::Capabilities IlbmPlugin::capabilities(QIODevice *device, const QByteArray &format) const
{
    if (format == "bbm" || format == "ilbm" || format == "iff" || format == "lbm") {
        return Capabilities(CanRead);
    }
    
    if (!format.isEmpty()) {
        return {};
    }
    
    if (!device->isOpen()) {
        return {};
    }
    
    Capabilities cap;
    if (device->isReadable() && IlbmHandler::canRead(device)) {
        cap |= CanRead;
    }
    return cap;
}

QImageIOHandler *IlbmPlugin::create(QIODevice *device, const QByteArray &format) const
{
    QImageIOHandler *handler = new IlbmHandler;
    handler->setDevice(device);
    handler->setFormat(format);
    return handler;
}
