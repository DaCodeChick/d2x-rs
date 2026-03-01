#include "core/io/DhfArchive.h"
#include <QCoreApplication>
#include <QCommandLineParser>
#include <QDir>
#include <print>
#include <cstdlib>

using namespace dle;

int main(int argc, char* argv[]) {
    QCoreApplication app(argc, argv);
    QCoreApplication::setApplicationName("dhfextract");
    QCoreApplication::setApplicationVersion("1.0.0");
    
    QCommandLineParser parser;
    parser.setApplicationDescription("DHF/HOG archive extraction tool for Descent 1 & 2");
    parser.addHelpOption();
    parser.addVersionOption();
    
    // Positional arguments
    parser.addPositionalArgument("archive", "Path to .hog/.sow/.mn2 file");
    
    // Options
    QCommandLineOption outputOption(
        {"o", "output"},
        "Output directory (default: current directory)",
        "directory",
        "."
    );
    parser.addOption(outputOption);
    
    QCommandLineOption listOption(
        {"l", "list"},
        "List files in archive without extracting"
    );
    parser.addOption(listOption);
    
    QCommandLineOption fileOption(
        {"f", "file"},
        "Extract specific file(s) only",
        "filename"
    );
    parser.addOption(fileOption);
    
    // Parse arguments
    parser.process(app);
    
    const QStringList args = parser.positionalArguments();
    if (args.isEmpty()) {
        std::println(stderr, "Error: No archive file specified");
        parser.showHelp(1);
    }
    
    QString archivePath = args.first();
    QString outputDir = parser.value(outputOption);
    bool listOnly = parser.isSet(listOption);
    QStringList filesToExtract = parser.values(fileOption);
    
    // Open archive
    std::println("Opening archive: {}", archivePath.toStdString());
    auto result = DhfArchive::open(archivePath);
    if (!result) {
        std::println(stderr, "Error: Failed to open archive: {}", 
                    dhfErrorString(result.error()).toStdString());
        return 1;
    }
    
    DhfArchive& archive = *result;
    std::println("Archive contains {} files", archive.fileCount());
    std::println("");
    
    // List mode
    if (listOnly) {
        std::println("{:<40} {:>10}", "Filename", "Size");
        std::println("{:-<40} {:->10}", "", "");
        
        uint64_t totalSize = 0;
        for (const auto& entry : archive.entries()) {
            std::println("{:<40} {:>10}", 
                        entry.name.toStdString(), 
                        entry.size);
            totalSize += entry.size;
        }
        
        std::println("");
        std::println("Total: {} files, {} bytes", archive.fileCount(), totalSize);
        return 0;
    }
    
    // Extract mode
    if (filesToExtract.isEmpty()) {
        // Extract all files
        std::println("Extracting all files to: {}", outputDir.toStdString());
        auto extractResult = archive.extractAll(outputDir);
        if (!extractResult) {
            std::println(stderr, "Error: Extraction failed: {}", 
                        dhfErrorString(extractResult.error()).toStdString());
            return 1;
        }
        std::println("");
        std::println("Successfully extracted {} files", archive.fileCount());
    } else {
        // Extract specific files
        std::println("Extracting {} file(s) to: {}", 
                    filesToExtract.size(), 
                    outputDir.toStdString());
        
        int successCount = 0;
        for (const QString& filename : filesToExtract) {
            QString destPath = QDir(outputDir).filePath(filename);
            auto extractResult = archive.extractFile(filename, destPath);
            if (extractResult) {
                std::println("Extracted: {}", filename.toStdString());
                successCount++;
            } else {
                std::println(stderr, "Failed to extract {}: {}", 
                            filename.toStdString(),
                            dhfErrorString(extractResult.error()).toStdString());
            }
        }
        
        std::println("");
        std::println("Successfully extracted {} of {} files", 
                    successCount, filesToExtract.size());
    }
    
    return 0;
}
