#include "IlbmHandler.h"
#include <QIODevice>
#include <QDataStream>
#include <QDebug>
#include <QImage>
#include <algorithm>
#include <format>

IlbmHandler::IlbmHandler()
    : m_headerRead(false)
{
}

bool IlbmHandler::canRead() const
{
    if (!m_headerRead && device()) {
        return canRead(device());
    }
    return m_headerRead;
}

bool IlbmHandler::canRead(QIODevice *device)
{
    if (!device) {
        qWarning("IlbmHandler::canRead() called with no device");
        return false;
    }
    
    // Check if we can peek at the FORM signature
    if (device->isSequential() || device->size() < 12) {
        return false;
    }
    
    // IFF files start with "FORM" signature
    char signature[4];
    if (device->peek(signature, 4) != 4) {
        return false;
    }
    
    return signature[0] == 'F' && 
           signature[1] == 'O' && 
           signature[2] == 'R' && 
           signature[3] == 'M';
}

std::expected<IlbmHandler::ChunkId, IlbmHandler::ParseError> IlbmHandler::readChunkId()
{
    auto bytes = readBytes(4);
    if (!bytes) {
        return std::unexpected(bytes.error());
    }
    
    ChunkId id;
    std::copy_n(bytes->constData(), 4, id.id.begin());
    return id;
}

std::expected<uint32_t, IlbmHandler::ParseError> IlbmHandler::readU32BE()
{
    auto bytes = readBytes(4);
    if (!bytes) {
        return std::unexpected(bytes.error());
    }
    
    const auto* data = reinterpret_cast<const uint8_t*>(bytes->constData());
    return (static_cast<uint32_t>(data[0]) << 24) |
           (static_cast<uint32_t>(data[1]) << 16) |
           (static_cast<uint32_t>(data[2]) << 8) |
           static_cast<uint32_t>(data[3]);
}

std::expected<uint16_t, IlbmHandler::ParseError> IlbmHandler::readU16BE()
{
    auto bytes = readBytes(2);
    if (!bytes) {
        return std::unexpected(bytes.error());
    }
    
    const auto* data = reinterpret_cast<const uint8_t*>(bytes->constData());
    return (static_cast<uint16_t>(data[0]) << 8) | static_cast<uint16_t>(data[1]);
}

std::expected<int16_t, IlbmHandler::ParseError> IlbmHandler::readI16BE()
{
    auto u16 = readU16BE();
    if (!u16) {
        return std::unexpected(u16.error());
    }
    return static_cast<int16_t>(*u16);
}

std::expected<uint8_t, IlbmHandler::ParseError> IlbmHandler::readU8()
{
    auto bytes = readBytes(1);
    if (!bytes) {
        return std::unexpected(bytes.error());
    }
    return static_cast<uint8_t>(bytes->at(0));
}

std::expected<QByteArray, IlbmHandler::ParseError> IlbmHandler::readBytes(qint64 count)
{
    QIODevice* dev = device();
    if (!dev) {
        return std::unexpected(ParseError{"No device available"});
    }
    
    QByteArray data = dev->read(count);
    if (data.size() != count) {
        return std::unexpected(ParseError{
            std::format("Failed to read {} bytes (got {})", count, data.size())
        });
    }
    return data;
}

std::expected<IlbmHandler::BitmapHeader, IlbmHandler::ParseError> IlbmHandler::parseBmhd()
{
    BitmapHeader header;
    
    auto width = readU16BE();
    if (!width) return std::unexpected(width.error());
    header.width = *width;
    
    auto height = readU16BE();
    if (!height) return std::unexpected(height.error());
    header.height = *height;
    
    auto x = readI16BE();
    if (!x) return std::unexpected(x.error());
    header.x = *x;
    
    auto y = readI16BE();
    if (!y) return std::unexpected(y.error());
    header.y = *y;
    
    auto bitPlanes = readU8();
    if (!bitPlanes) return std::unexpected(bitPlanes.error());
    header.bitPlanes = *bitPlanes;
    
    auto masking = readU8();
    if (!masking) return std::unexpected(masking.error());
    header.masking = static_cast<Masking>(*masking);
    
    auto compression = readU8();
    if (!compression) return std::unexpected(compression.error());
    header.compression = static_cast<Compression>(*compression);
    
    auto pad = readU8(); // padding byte
    if (!pad) return std::unexpected(pad.error());
    
    auto transparentColor = readU16BE();
    if (!transparentColor) return std::unexpected(transparentColor.error());
    header.transparentColor = *transparentColor;
    
    auto xAspect = readU8();
    if (!xAspect) return std::unexpected(xAspect.error());
    header.xAspect = *xAspect;
    
    auto yAspect = readU8();
    if (!yAspect) return std::unexpected(yAspect.error());
    header.yAspect = *yAspect;
    
    auto pageWidth = readI16BE();
    if (!pageWidth) return std::unexpected(pageWidth.error());
    header.pageWidth = static_cast<uint16_t>(*pageWidth);
    
    auto pageHeight = readI16BE();
    if (!pageHeight) return std::unexpected(pageHeight.error());
    header.pageHeight = static_cast<uint16_t>(*pageHeight);
    
    return header;
}

bool IlbmHandler::read(QImage *image)
{
    if (!image) {
        return false;
    }
    
    QIODevice* dev = device();
    if (!dev) {
        qWarning("IlbmHandler::read() called with no device");
        return false;
    }
    
    // Read FORM chunk
    auto formId = readChunkId();
    if (!formId || *formId != ChunkId::FORM()) {
        qWarning() << "Not an IFF file: expected FORM, got" 
                   << (formId ? formId->asString().c_str() : "error");
        return false;
    }
    
    auto formLength = readU32BE();
    if (!formLength) {
        qWarning("Failed to read FORM length");
        return false;
    }
    
    // Read form type (ILBM or PBM)
    auto formType = readChunkId();
    if (!formType) {
        qWarning("Failed to read form type");
        return false;
    }
    
    BitmapType bitmapType;
    if (*formType == ChunkId::ILBM()) {
        bitmapType = BitmapType::Ilbm;
    } else if (*formType == ChunkId::PBM()) {
        bitmapType = BitmapType::Pbm;
    } else {
        qWarning() << "Unknown IFF form type:" << formType->asString().c_str();
        return false;
    }
    
    std::optional<BitmapHeader> header;
    std::optional<QByteArray> palette;
    std::optional<QByteArray> compressedBody;
    
    qint64 formEnd = dev->pos() + *formLength - 4; // -4 because we already read form type
    
    // Parse chunks
    while (dev->pos() < formEnd) {
        auto chunkId = readChunkId();
        if (!chunkId) {
            qWarning("Failed to read chunk ID");
            return false;
        }
        
        auto chunkLen = readU32BE();
        if (!chunkLen) {
            qWarning("Failed to read chunk length");
            return false;
        }
        
        if (*chunkId == ChunkId::BMHD()) {
            auto bmhd = parseBmhd();
            if (!bmhd) {
                qWarning() << "Failed to parse BMHD:" << bmhd.error().message.c_str();
                return false;
            }
            header = *bmhd;
            m_header = *bmhd;
            m_size = QSize(bmhd->width, bmhd->height);
            m_headerRead = true;
        } else if (*chunkId == ChunkId::CMAP()) {
            auto data = readBytes(*chunkLen);
            if (!data) {
                qWarning() << "Failed to read CMAP data:" << data.error().message.c_str();
                return false;
            }
            palette = *data;
        } else if (*chunkId == ChunkId::BODY()) {
            auto data = readBytes(*chunkLen);
            if (!data) {
                qWarning() << "Failed to read BODY data:" << data.error().message.c_str();
                return false;
            }
            compressedBody = *data;
        } else {
            // Skip unknown chunk
            dev->seek(dev->pos() + *chunkLen);
        }
        
        // Chunks must be word-aligned (pad byte if odd length)
        if (*chunkLen & 1) {
            dev->read(1); // skip padding byte
        }
    }
    
    if (!header) {
        qWarning("IFF file missing BMHD chunk");
        return false;
    }
    
    if (!compressedBody) {
        qWarning("IFF file missing BODY chunk");
        return false;
    }
    
    // Decompress bitmap data
    auto bitmapData = decompressBody(*header, *compressedBody, bitmapType);
    if (!bitmapData) {
        qWarning() << "Failed to decompress bitmap data:" << bitmapData.error().message.c_str();
        return false;
    }
    
    // Create QImage
    *image = QImage(header->width, header->height, QImage::Format_Indexed8);
    
    // Set palette if present
    if (palette) {
        QVector<QRgb> colorTable;
        colorTable.reserve(256);
        
        const uint8_t* palData = reinterpret_cast<const uint8_t*>(palette->constData());
        size_t numColors = std::min<size_t>(palette->size() / 3, 256);
        
        for (size_t i = 0; i < numColors; ++i) {
            uint8_t r = palData[i * 3 + 0];
            uint8_t g = palData[i * 3 + 1];
            uint8_t b = palData[i * 3 + 2];
            colorTable.append(qRgb(r, g, b));
        }
        
        // Fill remaining colors with black
        while (colorTable.size() < 256) {
            colorTable.append(qRgb(0, 0, 0));
        }
        
        image->setColorTable(colorTable);
    }
    
    // Copy pixel data
    for (int y = 0; y < header->height; ++y) {
        uint8_t* scanLine = image->scanLine(y);
        const uint8_t* srcLine = bitmapData->data() + y * header->width;
        std::copy_n(srcLine, header->width, scanLine);
    }
    
    return true;
}

bool IlbmHandler::write(const QImage &image)
{
    Q_UNUSED(image);
    return false; // Writing not supported
}

QVariant IlbmHandler::option(ImageOption option) const
{
    if (option == Size && m_headerRead) {
        return m_size;
    }
    return QVariant();
}

void IlbmHandler::setOption(ImageOption option, const QVariant &value)
{
    Q_UNUSED(option);
    Q_UNUSED(value);
}

bool IlbmHandler::supportsOption(ImageOption option) const
{
    return option == Size;
}

std::expected<std::vector<uint8_t>, IlbmHandler::ParseError> 
IlbmHandler::decompressBody(
    const BitmapHeader& header, 
    const QByteArray& compressed, 
    BitmapType bitmapType)
{
    size_t width = (bitmapType == BitmapType::Pbm) ? header.width : ((header.width + 15) / 16) * 2;
    size_t depth = (bitmapType == BitmapType::Pbm) ? 1 : header.bitPlanes;
    
    std::vector<uint8_t> decompressed;
    
    if (header.compression == Compression::None) {
        auto result = decompressUncompressed(compressed, header, width, depth);
        if (!result) return std::unexpected(result.error());
        decompressed = std::move(*result);
    } else {
        size_t rowSize = width * depth;
        size_t totalSize = rowSize * header.height;
        auto result = decompressByteRun1(compressed, header, width, depth, totalSize);
        if (!result) return std::unexpected(result.error());
        decompressed = std::move(*result);
    }
    
    // Convert ILBM planar format to chunky 8-bit indexed
    if (bitmapType == BitmapType::Ilbm) {
        return convertIlbmToChunky(decompressed, header.width, header.height, header.bitPlanes);
    }
    
    return decompressed;
}

std::expected<std::vector<uint8_t>, IlbmHandler::ParseError>
IlbmHandler::decompressUncompressed(
    const QByteArray& data, 
    const BitmapHeader& header, 
    size_t width, 
    size_t depth)
{
    size_t rowSize = width * depth;
    size_t totalSize = rowSize * header.height;
    std::vector<uint8_t> output;
    output.reserve(totalSize);
    
    size_t pos = 0;
    const uint8_t* bytes = reinterpret_cast<const uint8_t*>(data.constData());
    
    for (uint16_t row = 0; row < header.height; ++row) {
        if (pos + rowSize > static_cast<size_t>(data.size())) {
            return std::unexpected(ParseError{"Truncated IFF body data"});
        }
        
        output.insert(output.end(), bytes + pos, bytes + pos + rowSize);
        pos += rowSize;
        
        // Skip mask data if present
        if (header.masking == Masking::HasMask) {
            pos += width;
        }
        
        // Skip padding byte for odd width
        if (header.width & 1) {
            pos += 1;
        }
    }
    
    return output;
}

std::expected<std::vector<uint8_t>, IlbmHandler::ParseError>
IlbmHandler::decompressByteRun1(
    const QByteArray& data, 
    const BitmapHeader& header, 
    size_t width, 
    size_t depth, 
    size_t totalSize)
{
    std::vector<uint8_t> output;
    output.reserve(totalSize);
    
    size_t pos = 0;
    size_t widCnt = width;
    size_t plane = 0;
    ssize_t endCnt = (width & 1) ? -1 : 0;
    
    const uint8_t* bytes = reinterpret_cast<const uint8_t*>(data.constData());
    
    while (pos < static_cast<size_t>(data.size()) && output.size() < totalSize) {
        if (static_cast<ssize_t>(widCnt) == endCnt) {
            widCnt = width;
            plane += 1;
            if ((header.masking == Masking::HasMask && plane == depth + 1) ||
                (header.masking != Masking::HasMask && plane == depth)) {
                plane = 0;
            }
        }
        
        int8_t n = static_cast<int8_t>(bytes[pos++]);
        
        if (n >= 0) {
            // Copy next n+1 bytes literally
            size_t count = static_cast<size_t>(n) + 1;
            
            if (widCnt < count) {
                count = widCnt + 1;
            }
            widCnt -= count;
            
            size_t actualCount = count;
            if (widCnt == SIZE_MAX) { // wrapped
                actualCount -= 1;
                widCnt = 0;
            }
            
            if (plane == depth) {
                // Skip mask data
                pos += actualCount;
            } else {
                if (pos + actualCount > static_cast<size_t>(data.size())) {
                    return std::unexpected(ParseError{"Truncated RLE data"});
                }
                output.insert(output.end(), bytes + pos, bytes + pos + actualCount);
                pos += actualCount;
            }
            
            if (actualCount != count) {
                pos += 1; // skip padding
            }
        } else if (n > -128) {
            // Repeat next byte -n+1 times
            size_t count = static_cast<size_t>(-n) + 1;
            
            if (pos >= static_cast<size_t>(data.size())) {
                return std::unexpected(ParseError{"Truncated RLE data"});
            }
            uint8_t byte = bytes[pos++];
            
            if (widCnt < count) {
                count = widCnt + 1;
            }
            widCnt -= count;
            
            size_t actualCount = count;
            if (widCnt == SIZE_MAX) {
                actualCount -= 1;
                widCnt = 0;
            }
            
            if (plane != depth) {
                // Not mask data
                output.insert(output.end(), actualCount, byte);
            }
        }
        // n == -128 is a no-op
    }
    
    if (output.size() != totalSize) {
        return std::unexpected(ParseError{
            std::format("IFF decompression size mismatch: expected {}, got {}", 
                       totalSize, output.size())
        });
    }
    
    return output;
}

std::vector<uint8_t> IlbmHandler::convertIlbmToChunky(
    const std::vector<uint8_t>& planar, 
    uint16_t width, 
    uint16_t height, 
    uint8_t bitPlanes)
{
    size_t w = width;
    size_t h = height;
    size_t planes = bitPlanes;
    size_t bytesPerRow = (w + 7) / 8;
    
    std::vector<uint8_t> chunky(w * h, 0);
    
    for (size_t y = 0; y < h; ++y) {
        for (size_t x = 0; x < w; ++x) {
            uint8_t pixel = 0;
            size_t byteX = x / 8;
            size_t bitX = 7 - (x % 8);
            
            for (size_t plane = 0; plane < planes; ++plane) {
                size_t offset = (y * planes * bytesPerRow) + (plane * bytesPerRow) + byteX;
                if (offset < planar.size()) {
                    uint8_t byte = planar[offset];
                    uint8_t bit = (byte >> bitX) & 1;
                    pixel |= bit << plane;
                }
            }
            
            chunky[y * w + x] = pixel;
        }
    }
    
    return chunky;
}
