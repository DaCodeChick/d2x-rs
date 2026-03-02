#pragma once

#include <QImageIOHandler>
#include <QImage>
#include <array>
#include <cstdint>
#include <expected>
#include <string>
#include <vector>

/**
 * @brief Qt image I/O handler for IFF ILBM/PBM format
 * 
 * Supports reading IFF (Interchange File Format) images used in Descent 1 & 2
 * for briefing screens. The format originates from Commodore Amiga.
 * Handles:
 * - ILBM (Interleaved Bitmap) - bitplane format
 * - PBM (Planar Bitmap) - 8-bit indexed format
 * - ByteRun1 RLE compression
 * - 256-color palettes (CMAP chunk)
 * 
 * These files typically have a `.bbm` extension in Descent.
 * Writing is not supported as IFF is a legacy read-only format.
 */
class IlbmHandler : public QImageIOHandler
{
public:
    IlbmHandler();
    
    bool canRead() const override;
    bool read(QImage *image) override;
    bool write(const QImage &image) override;
    
    QVariant option(ImageOption option) const override;
    void setOption(ImageOption option, const QVariant &value) override;
    bool supportsOption(ImageOption option) const override;
    
    static bool canRead(QIODevice *device);
    
private:
    /// IFF chunk identifier (4-byte signature)
    struct ChunkId {
        std::array<char, 4> id;
        
        constexpr bool operator==(const ChunkId& other) const {
            return id == other.id;
        }
        
        std::string asString() const {
            return std::string(id.data(), 4);
        }
        
        static constexpr ChunkId FORM() { return {{'F', 'O', 'R', 'M'}}; }
        static constexpr ChunkId ILBM() { return {{'I', 'L', 'B', 'M'}}; }
        static constexpr ChunkId PBM()  { return {{'P', 'B', 'M', ' '}}; }
        static constexpr ChunkId BMHD() { return {{'B', 'M', 'H', 'D'}}; }
        static constexpr ChunkId CMAP() { return {{'C', 'M', 'A', 'P'}}; }
        static constexpr ChunkId BODY() { return {{'B', 'O', 'D', 'Y'}}; }
    };
    
    /// Bitmap type
    enum class BitmapType : uint8_t {
        Pbm,   ///< Planar bitmap (8-bit indexed)
        Ilbm   ///< Interleaved bitmap (bitplanes)
    };
    
    /// Compression method
    enum class Compression : uint8_t {
        None = 0,      ///< No compression
        ByteRun1 = 1   ///< ByteRun1 RLE compression
    };
    
    /// Masking type
    enum class Masking : uint8_t {
        None = 0,            ///< No masking
        HasMask = 1,         ///< Has mask plane
        TransparentColor = 2 ///< Has transparent color
    };
    
    /// Bitmap header (BMHD chunk)
    struct BitmapHeader {
        uint16_t width{0};           ///< Image width in pixels
        uint16_t height{0};          ///< Image height in pixels
        int16_t x{0};                ///< X position (usually 0)
        int16_t y{0};                ///< Y position (usually 0)
        uint8_t bitPlanes{0};        ///< Number of bit planes
        Masking masking{Masking::None}; ///< Masking type
        Compression compression{Compression::None}; ///< Compression method
        uint16_t transparentColor{0}; ///< Transparent color index
        uint8_t xAspect{5};          ///< X aspect ratio
        uint8_t yAspect{6};          ///< Y aspect ratio
        uint16_t pageWidth{0};       ///< Page width
        uint16_t pageHeight{0};      ///< Page height
    };
    
    /// Error type for parsing operations
    struct ParseError {
        std::string message;
    };
    
    // Parsing methods
    std::expected<ChunkId, ParseError> readChunkId();
    std::expected<uint32_t, ParseError> readU32BE();
    std::expected<uint16_t, ParseError> readU16BE();
    std::expected<int16_t, ParseError> readI16BE();
    std::expected<uint8_t, ParseError> readU8();
    std::expected<QByteArray, ParseError> readBytes(qint64 count);
    
    std::expected<BitmapHeader, ParseError> parseBmhd();
    std::expected<std::vector<uint8_t>, ParseError> decompressBody(
        const BitmapHeader& header, 
        const QByteArray& compressed, 
        BitmapType bitmapType
    );
    std::expected<std::vector<uint8_t>, ParseError> decompressUncompressed(
        const QByteArray& data, 
        const BitmapHeader& header, 
        size_t width, 
        size_t depth
    );
    std::expected<std::vector<uint8_t>, ParseError> decompressByteRun1(
        const QByteArray& data, 
        const BitmapHeader& header, 
        size_t width, 
        size_t depth, 
        size_t totalSize
    );
    std::vector<uint8_t> convertIlbmToChunky(
        const std::vector<uint8_t>& planar, 
        uint16_t width, 
        uint16_t height, 
        uint8_t bitPlanes
    );
    
    BitmapHeader m_header;
    bool m_headerRead{false};
    QSize m_size;
};
