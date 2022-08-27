#include "mainwindow.h"
#include "ui_mainwindow.h"
#include <QGridLayout>
#include <QLabel>
#include <QCheckBox>
#include <QProcess>
#include <QMessageBox>
#include <QDebug>

const int MAX_NUMBER = 6;

MainWindow::MainWindow(QWidget *parent)
    : QMainWindow(parent)
    , ui(new Ui::MainWindow)
{
    ui->setupUi(this);
    setWindowTitle(QStringLiteral("方块解迷"));
    setWindowIcon(QIcon(":app.png"));
}

MainWindow::~MainWindow()
{
    delete ui;
}

QVector<bool> MainWindow::generate_index_map(QGroupBox * group_box, int number)
{
    QVector<bool> ret(number, false);

    for (auto obj: group_box->children()) {
        QString name = obj->objectName();

        if (name.startsWith("m_cb")) {
            int index = name[name.length() - 1].digitValue() - 1;

            if (index < ret.length()) {
                if (((QCheckBox*)obj)->isChecked()) {
                    ret[index] = true;
                }
            }
        }
    }

    return ret;
}

bool MainWindow::check_units_state(QLineEdit *line_edit, int number)
{
    QString text = line_edit->text();

    if (!text.isEmpty()) {
        QStringList values = text.split(',');

        if (values.length() != number) {
            QString msg{};

            msg += "'";
            msg += text;
            msg += "'";
            msg += QStringLiteral(" 状态个数和方块不匹配");
            QMessageBox::warning(this, QStringLiteral("警告"), msg);
            return false;
        }
        return true;
    }
    else {
        return false;
    }
}

QString generate_units_state(QString text) {
        QVector<int> states{};
        QStringList values = text.split(',');

        for (auto & value: values) {
           int state = value.toInt();

           states.push_back(state - 1);
        }
        QString ret{};

        for (auto state: states) {
            ret += QString::number(state);
            ret += ",";
        }
        return ret;
}

void MainWindow::on_pushButton_clicked()
{
    int number = ui->m_sb_number->value();
    int max_state = ui->m_sb_max_state->value();

    QVector<bool> maps[MAX_NUMBER] = {
        generate_index_map(ui->m_gb_units1, number),
        generate_index_map(ui->m_gb_units2, number),
        generate_index_map(ui->m_gb_units3, number),
        generate_index_map(ui->m_gb_units4, number),
        generate_index_map(ui->m_gb_units5, number),
        generate_index_map(ui->m_gb_units6, number),
   };

    QProcess process{};
    QStringList arguments{};

    arguments.push_back(QString("-N"));
    arguments.push_back(QString::number(number));
    arguments.push_back(QString("-M"));
    arguments.push_back(QString::number(max_state));
    for (int i = 0;i < number; i++) {
        QString current = QString::number(i);

        current += ":";
        for (int j = 0;j < number; j ++) {
            if (maps[i][j]) {
                current += QString::number(j);
                current += ",";
            }
        }

        arguments.push_back(QString("-L"));
        arguments.push_back(current);
    }
    if (check_units_state(ui->m_le_beg, number)) {
        arguments.push_back(QString("-B"));
        arguments.push_back(generate_units_state(ui->m_le_beg->text()));
    }
    if (check_units_state(ui->m_le_end1, number)) {
        arguments.push_back(QString("-E"));
        arguments.push_back(generate_units_state(ui->m_le_end1->text()));
    }
    if (check_units_state(ui->m_le_end2, number)) {
        arguments.push_back(QString("-E"));
        arguments.push_back(generate_units_state(ui->m_le_end2->text()));
    }
    if (check_units_state(ui->m_le_end3, number)) {
        arguments.push_back(QString("-E"));
        arguments.push_back(generate_units_state(ui->m_le_end3->text()));
    }
    if (check_units_state(ui->m_le_end4, number)) {
        arguments.push_back(QString("-E"));
        arguments.push_back(generate_units_state(ui->m_le_end4->text()));
    }

    qDebug() << arguments << Qt::endl;

    process.start("yuanshen.exe", arguments);

    if (process.waitForStarted()) {
        if (process.waitForReadyRead()) {
            ui->m_output->setText(QString(process.readAllStandardOutput()));
        }
    }
    else {
        QMessageBox::warning(this, QStringLiteral("警告"), QStringLiteral("无法启动yuanshen.exe"));
    }
}



