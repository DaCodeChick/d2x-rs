#ifndef DLE_MAINWINDOW_H
#define DLE_MAINWINDOW_H

#include <QMainWindow>
#include <memory>
#include "core/mine/Mine.h"

QT_BEGIN_NAMESPACE
namespace Ui {
class MainWindow;
}
QT_END_NAMESPACE

namespace dle {

/**
 * @brief Main window for the Descent Level Editor
 * 
 * Features:
 * - Menu bar (File, Edit, View, Tools, Help)
 * - Tool palette for editing operations
 * - 3D viewport for level visualization
 * - Property panels for selected objects
 * - Status bar
 */
class MainWindow : public QMainWindow {
    Q_OBJECT

public:
    explicit MainWindow(QWidget *parent = nullptr);
    ~MainWindow();

private slots:
    // File menu
    void onFileNew();
    void onFileOpen();
    bool onFileSave();
    bool onFileSaveAs();
    void onFileExit();
    
    // Edit menu
    void onEditUndo();
    void onEditRedo();
    void onEditCut();
    void onEditCopy();
    void onEditPaste();
    void onEditDelete();
    
    // View menu
    void onViewWireframe();
    void onViewTextured();
    void onViewLighting();
    void onToggleTexturePalette();
    void onToggleTextureBar();
    void onToggleProperties();
    void onToggleSegmentInfo();
    
    // Help menu
    void onHelpAbout();

private:
    void setupConnections();
    void updateWindowTitle();
    void newLevel();
    bool openLevel(const QString& filename);
    bool saveLevel(const QString& filename);
    bool maybeSave();
    
    std::unique_ptr<Ui::MainWindow> ui;
    std::unique_ptr<Mine> m_mine;
    QString m_currentFilename;
};

} // namespace dle

#endif // DLE_MAINWINDOW_H
