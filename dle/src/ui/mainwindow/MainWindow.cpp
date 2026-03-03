#include "MainWindow.h"
#include "ui_MainWindow.h"
#include "../toolpanel/ToolPanel.h"
#include <QMessageBox>
#include <QFileDialog>

namespace dle {

MainWindow::MainWindow(QWidget *parent)
    : QMainWindow(parent)
    , ui(std::make_unique<Ui::MainWindow>())
    , m_mine(std::make_unique<Mine>())
    , m_toolPanel(nullptr)
{
    ui->setupUi(this);
    setupConnections();
    setupDockWidgets();
    
    // Create a new default level
    newLevel();
    updateWindowTitle();
}

MainWindow::~MainWindow() = default;

void MainWindow::setupConnections() {
    // File menu
    connect(ui->actionNew, &QAction::triggered, this, &MainWindow::onFileNew);
    connect(ui->actionOpen, &QAction::triggered, this, &MainWindow::onFileOpen);
    connect(ui->actionSave, &QAction::triggered, this, &MainWindow::onFileSave);
    connect(ui->actionSaveAs, &QAction::triggered, this, &MainWindow::onFileSaveAs);
    connect(ui->actionExit, &QAction::triggered, this, &MainWindow::onFileExit);
    
    // Edit menu
    connect(ui->actionUndo, &QAction::triggered, this, &MainWindow::onEditUndo);
    connect(ui->actionRedo, &QAction::triggered, this, &MainWindow::onEditRedo);
    connect(ui->actionCut, &QAction::triggered, this, &MainWindow::onEditCut);
    connect(ui->actionCopy, &QAction::triggered, this, &MainWindow::onEditCopy);
    connect(ui->actionPaste, &QAction::triggered, this, &MainWindow::onEditPaste);
    connect(ui->actionDelete, &QAction::triggered, this, &MainWindow::onEditDelete);
    
    // View menu
    connect(ui->actionWireframe, &QAction::triggered, this, &MainWindow::onViewWireframe);
    connect(ui->actionTextured, &QAction::triggered, this, &MainWindow::onViewTextured);
    connect(ui->actionLighting, &QAction::triggered, this, &MainWindow::onViewLighting);
    connect(ui->actionToggleTexturePalette, &QAction::triggered, this, &MainWindow::onToggleTexturePalette);
    connect(ui->actionToggleTextureBar, &QAction::triggered, this, &MainWindow::onToggleTextureBar);
    connect(ui->actionToggleProperties, &QAction::triggered, this, &MainWindow::onToggleProperties);
    connect(ui->actionToggleSegmentInfo, &QAction::triggered, this, &MainWindow::onToggleSegmentInfo);
    
    // Help menu
    connect(ui->actionAbout, &QAction::triggered, this, &MainWindow::onHelpAbout);
}

void MainWindow::setupDockWidgets() {
    // Create tool panel dock widget
    m_toolPanel = new ToolPanel(this);
    addDockWidget(Qt::LeftDockWidgetArea, m_toolPanel);
    
    // Connect tool panel visibility to menu action if it exists
    // Note: We'll need to add actionToggleToolPanel to the UI file later
    // For now, the tool panel is visible by default
    
    // Connect tool panel to mine data
    if (m_mine) {
        m_toolPanel->setMine(m_mine.get());
    }
}

void MainWindow::updateWindowTitle() {
    QString title = "DLE - Descent Level Editor";
    if (!m_currentFilename.isEmpty()) {
        title += " - " + m_currentFilename;
    }
    if (m_mine && m_mine->hasUnsavedChanges()) {
        title += " *";
    }
    setWindowTitle(title);
}

void MainWindow::newLevel() {
    if (!maybeSave()) {
        return;
    }
    
    m_mine->createDefault();
    m_currentFilename.clear();
    updateWindowTitle();
    
    // Update viewport with new mine
    if (ui->viewport) {
        ui->viewport->setMine(m_mine.get());
    }
    
    // Update tool panel with new mine
    if (m_toolPanel) {
        m_toolPanel->setMine(m_mine.get());
    }
    
    ui->statusbar->showMessage("New level created", 2000);
}

bool MainWindow::openLevel(const QString& filename) {
    if (!m_mine->load(filename.toStdString())) {
        QMessageBox::critical(this, "Error", "Failed to load level: " + filename);
        return false;
    }
    
    m_currentFilename = filename;
    updateWindowTitle();
    
    // Update viewport with loaded mine
    if (ui->viewport) {
        ui->viewport->setMine(m_mine.get());
    }
    
    // Update tool panel with loaded mine
    if (m_toolPanel) {
        m_toolPanel->setMine(m_mine.get());
    }
    
    ui->statusbar->showMessage("Loaded: " + filename, 2000);
    return true;
}

bool MainWindow::saveLevel(const QString& filename) {
    if (!m_mine->save(filename.toStdString())) {
        QMessageBox::critical(this, "Error", "Failed to save level: " + filename);
        return false;
    }
    
    m_currentFilename = filename;
    m_mine->markSaved();
    updateWindowTitle();
    ui->statusbar->showMessage("Saved: " + filename, 2000);
    return true;
}

bool MainWindow::maybeSave() {
    if (!m_mine || !m_mine->hasUnsavedChanges()) {
        return true;
    }
    
    QMessageBox::StandardButton ret = QMessageBox::question(
        this,
        "Unsaved Changes",
        "The level has unsaved changes. Do you want to save them?",
        QMessageBox::Save | QMessageBox::Discard | QMessageBox::Cancel
    );
    
    if (ret == QMessageBox::Save) {
        return onFileSave();
    } else if (ret == QMessageBox::Cancel) {
        return false;
    }
    
    return true;
}

// File menu slots
void MainWindow::onFileNew() {
    newLevel();
}

void MainWindow::onFileOpen() {
    if (!maybeSave()) {
        return;
    }
    
    QString filename = QFileDialog::getOpenFileName(
        this,
        "Open Level",
        QString(),
        "Descent Levels (*.rdl *.rl2);;All Files (*)"
    );
    
    if (!filename.isEmpty()) {
        openLevel(filename);
    }
}

bool MainWindow::onFileSave() {
    if (m_currentFilename.isEmpty()) {
        return onFileSaveAs();
    }
    
    return saveLevel(m_currentFilename);
}

bool MainWindow::onFileSaveAs() {
    QString filename = QFileDialog::getSaveFileName(
        this,
        "Save Level",
        QString(),
        "Descent 2 Level (*.rl2);;Descent 1 Level (*.rdl);;All Files (*)"
    );
    
    if (filename.isEmpty()) {
        return false;
    }
    
    return saveLevel(filename);
}

void MainWindow::onFileExit() {
    close();
}

// Edit menu slots
void MainWindow::onEditUndo() {
    // TODO: Implement undo
    ui->statusbar->showMessage("Undo not yet implemented", 2000);
}

void MainWindow::onEditRedo() {
    // TODO: Implement redo
    ui->statusbar->showMessage("Redo not yet implemented", 2000);
}

void MainWindow::onEditCut() {
    // TODO: Implement cut
    ui->statusbar->showMessage("Cut not yet implemented", 2000);
}

void MainWindow::onEditCopy() {
    // TODO: Implement copy
    ui->statusbar->showMessage("Copy not yet implemented", 2000);
}

void MainWindow::onEditPaste() {
    // TODO: Implement paste
    ui->statusbar->showMessage("Paste not yet implemented", 2000);
}

void MainWindow::onEditDelete() {
    // TODO: Implement delete
    ui->statusbar->showMessage("Delete not yet implemented", 2000);
}

// View menu slots
void MainWindow::onViewWireframe() {
    if (ui->viewport) {
        bool enabled = ui->actionWireframe->isChecked();
        ui->viewport->setWireframeMode(enabled);
        ui->statusbar->showMessage(enabled ? "Wireframe view enabled" : "Wireframe view disabled", 2000);
    }
}

void MainWindow::onViewTextured() {
    if (ui->viewport) {
        bool enabled = ui->actionTextured->isChecked();
        ui->viewport->setWireframeMode(!enabled); // Textured = not wireframe
        ui->statusbar->showMessage(enabled ? "Textured view enabled" : "Textured view disabled", 2000);
    }
}

void MainWindow::onViewLighting() {
    // TODO: Implement lighting view
    ui->statusbar->showMessage("Lighting toggle not yet implemented", 2000);
}

// Help menu slots
void MainWindow::onHelpAbout() {
    QMessageBox::about(
        this,
        "About DLE",
        "<h2>DLE - Descent Level Editor</h2>"
        "<p>Version 1.0.0</p>"
        "<p>A modern cross-platform level editor for Descent 1 and Descent 2.</p>"
        "<p>Built with Qt 6 and C++23</p>"
        "<p>Segments: " + QString::number(m_mine->getSegmentCount()) + "</p>"
        "<p>Vertices: " + QString::number(m_mine->getVertexCount()) + "</p>"
    );
}

void MainWindow::onToggleTexturePalette() {
    bool visible = ui->actionToggleTexturePalette->isChecked();
    ui->texturePaletteDock->setVisible(visible);
    ui->statusbar->showMessage(visible ? "Texture palette shown" : "Texture palette hidden", 2000);
}

void MainWindow::onToggleTextureBar() {
    bool visible = ui->actionToggleTextureBar->isChecked();
    ui->textureBarDock->setVisible(visible);
    ui->statusbar->showMessage(visible ? "Texture bar shown" : "Texture bar hidden", 2000);
}

void MainWindow::onToggleProperties() {
    bool visible = ui->actionToggleProperties->isChecked();
    ui->propertiesDock->setVisible(visible);
    ui->statusbar->showMessage(visible ? "Properties panel shown" : "Properties panel hidden", 2000);
}

void MainWindow::onToggleSegmentInfo() {
    bool visible = ui->actionToggleSegmentInfo->isChecked();
    ui->segmentInfoDock->setVisible(visible);
    ui->statusbar->showMessage(visible ? "Segment info shown" : "Segment info hidden", 2000);
}

void MainWindow::onToggleToolPanel() {
    if (m_toolPanel) {
        bool visible = !m_toolPanel->isVisible();
        m_toolPanel->setVisible(visible);
        ui->statusbar->showMessage(visible ? "Tool panel shown" : "Tool panel hidden", 2000);
    }
}

} // namespace dle
