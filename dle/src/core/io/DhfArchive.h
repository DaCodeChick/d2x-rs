#ifndef DLE_DHFARCHIVE_H
#define DLE_DHFARCHIVE_H

#include <QFile>
#include <QString>
#include <QMap>
#include <QByteArray>
#include <expected>
#include <cstdint>

namespace dle {

/**
 * @brief Error types for DHF archive operations
 */
enum class DhfError {
    FileNotFound,
    IoError,
    InvalidFormat,
    EntryNotFound,
    CorruptEntry
};

/**
 * @brief Entry in a DHF archive
 */
struct DhfEntry {
    QString name;       ///< Filename (uppercase, case-insensitive)
    uint64_t offset;    ///< Offset in the archive file
    uint32_t size;      ///< Size in bytes
};

/**
 * @brief DHF archive file format parser (Descent 1 & 2)
 * 
 * DHF files (.hog, .sow, .mn2) are simple archive formats used by Descent 1 and 2
 * to package game assets. They contain no compression, only concatenation.
 * 
 * ## File Format
 * ```
 * [Optional Header: "DHF"]
 * For each file:
 *   - filename: 13 bytes (null-terminated)
 *   - size: u32 (little-endian)
 *   - data: [size] bytes
 * ```
 * 
 * ## Usage
 * ```cpp
 * auto result = DhfArchive::open("descent2.hog");
 * if (result) {
 *     DhfArchive& archive = *result;
 *     
 *     // List files
 *     for (const auto& entry : archive.entries()) {
 *         std::println("{}: {} bytes", entry.name, entry.size);
 *     }
 *     
 *     // Extract a file
 *     if (auto data = archive.readFile("LEVEL01.RL2")) {
 *         // Process data
 *     }
 * }
 * ```
 */
class DhfArchive {
public:
    /**
     * @brief Open a DHF archive file
     * 
     * @param path Path to the .hog/.sow file
     * @return Expected DhfArchive or DhfError
     */
    static std::expected<DhfArchive, DhfError> open(const QString& path);
    
    /**
     * @brief Check if a file exists in the archive
     * 
     * File names are case-insensitive.
     * 
     * @param name Filename to check
     * @return true if file exists
     */
    bool containsFile(const QString& name) const;
    
    /**
     * @brief Get entry information for a file
     * 
     * @param name Filename (case-insensitive)
     * @return Pointer to entry or nullptr if not found
     */
    const DhfEntry* getEntry(const QString& name) const;
    
    /**
     * @brief Read a file from the archive
     * 
     * @param name Filename (case-insensitive)
     * @return File data or DhfError
     */
    std::expected<QByteArray, DhfError> readFile(const QString& name);
    
    /**
     * @brief Extract a file to disk
     * 
     * @param name Filename in archive (case-insensitive)
     * @param destPath Destination file path
     * @return void or DhfError
     */
    std::expected<void, DhfError> extractFile(const QString& name, const QString& destPath);
    
    /**
     * @brief Extract all files to a directory
     * 
     * @param destDir Destination directory path
     * @return void or DhfError
     */
    std::expected<void, DhfError> extractAll(const QString& destDir);
    
    /**
     * @brief Get number of files in archive
     */
    int fileCount() const { return m_entries.size(); }
    
    /**
     * @brief Get all entries
     */
    QList<DhfEntry> entries() const { return m_entries.values(); }
    
    /**
     * @brief Get the archive file path
     */
    QString filePath() const { return m_filePath; }

private:
    explicit DhfArchive(QString filePath, QMap<QString, DhfEntry> entries)
        : m_filePath(std::move(filePath)), m_entries(std::move(entries)) {}
    
    /**
     * @brief Parse DHF format entries
     */
    static std::expected<QMap<QString, DhfEntry>, DhfError> parseEntries(QFile& file);
    
    QString m_filePath;
    QMap<QString, DhfEntry> m_entries;
};

/**
 * @brief Convert DhfError to human-readable string
 */
inline QString dhfErrorString(DhfError error) {
    switch (error) {
        case DhfError::FileNotFound: return "File not found";
        case DhfError::IoError: return "I/O error";
        case DhfError::InvalidFormat: return "Invalid format";
        case DhfError::EntryNotFound: return "Entry not found";
        case DhfError::CorruptEntry: return "Corrupt entry";
        default: return "Unknown error";
    }
}

} // namespace dle

#endif // DLE_DHFARCHIVE_H
