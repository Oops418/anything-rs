#[cfg(feature = "mock")]
mod mock_tests {
    use indexify::{
        get_num_docs, index_add, index_commit, index_delete, index_files, index_search,
    };
    use std::{fs, thread};
    use tempfile::TempDir;
    use vaultify::Vaultify;

    #[test]
    fn test_workflow() {
        Vaultify::init_vault();

        let (temp_folder, folder_num) =
            generate_mock_files().expect("Failed to generate mock files");
        let temp_folder = temp_folder.path().to_str().unwrap();
        assert_eq!(folder_num, 76);
        assert!(!temp_folder.is_empty());

        let remain_exclude_path: Vec<String> = vec!["None".to_string()];

        index_files(temp_folder, &remain_exclude_path);
        thread::sleep(std::time::Duration::from_millis(500));

        assert_eq!(get_num_docs(), 77);

        let delete_path1 = format!("{}/{}", temp_folder, "购物 清单 超市.md");
        let delete_path2 = format!("{}/{}", temp_folder, "会议纪要_Meeting_Notes.txt");
        let delete_path3 = format!("{}/{}", temp_folder, "化学-课件-第三章.pptx");
        let delete_path4 = format!("{}/{}", temp_folder, "测试报告_Test_Report.xml");

        index_delete(delete_path1.as_str()).unwrap();
        index_commit().unwrap();
        thread::sleep(std::time::Duration::from_millis(500));
        assert_eq!(get_num_docs(), 76);

        index_delete(delete_path2.as_str()).unwrap();
        index_commit().unwrap();
        thread::sleep(std::time::Duration::from_millis(500));
        assert_eq!(get_num_docs(), 75);

        index_delete(delete_path3.as_str()).unwrap();
        index_commit().unwrap();
        thread::sleep(std::time::Duration::from_millis(500));
        assert_eq!(get_num_docs(), 74);

        index_delete(delete_path4.as_str()).unwrap();
        index_commit().unwrap();
        thread::sleep(std::time::Duration::from_millis(500));
        assert_eq!(get_num_docs(), 73);

        let search_results = index_search("生产");
        assert!(!search_results.is_empty());

        let search_results = index_search("Draft");
        assert!(!search_results.is_empty());

        index_add(format!("{}/{}", temp_folder, "原神.pdf").as_str()).unwrap();
        index_add(format!("{}/{}", temp_folder, "genshin.pdf").as_str()).unwrap();
        index_commit().unwrap();
        thread::sleep(std::time::Duration::from_millis(500));
        assert_eq!(get_num_docs(), 75);

        let search_results = index_search("genshin");
        assert_eq!(
            search_results.get(0).unwrap().path,
            format!("{}/{}", temp_folder, "genshin.pdf")
        );

        let search_results = index_search("原神");
        assert_eq!(
            search_results.get(0).unwrap().path,
            format!("{}/{}", temp_folder, "原神.pdf")
        );

        let dupicate_path = format!("{}/{}", temp_folder, "rust_duplicate.pdf");
        index_add(dupicate_path.as_str()).unwrap();
        index_commit().unwrap();

        index_add(dupicate_path.as_str()).unwrap();
        index_commit().unwrap();
        thread::sleep(std::time::Duration::from_millis(500));
        assert_eq!(get_num_docs(), 77);

        index_delete(dupicate_path.as_str()).unwrap();
        index_commit().unwrap();
        thread::sleep(std::time::Duration::from_millis(500));
        assert_eq!(get_num_docs(), 75);
    }

    fn generate_mock_files() -> Result<(TempDir, usize), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;

        // English files - different domains and separators
        let english_files = vec![
            // Programming/Tech (underscore separator)
            "user_authentication.py",
            "database_connection_pool.js",
            "api_response_handler.ts",
            "unit_test_results.xml",
            "deployment_config.yaml",
            "error_log_analyzer.rb",
            // Business (dash separator)
            "quarterly-sales-report.xlsx",
            "client-meeting-notes.docx",
            "marketing-campaign-analysis.pdf",
            "financial-budget-2025.csv",
            "hr-policy-updates.txt",
            "project-timeline-overview.pptx",
            // Academic/Research (space separator - encoded as %20 or replaced)
            "Research Methodology.pdf",
            "Literature Review Draft.docx",
            "Experiment Data Analysis.xlsx",
            "Conference Presentation.pptx",
            "Thesis Chapter 3.txt",
            // Creative/Media (mixed separators)
            "PhotoShoot_Beach2025.jpg",
            "video-editing-project.mp4",
            "Logo Design v2.ai",
            "music_composition_draft.mp3",
            "website-mockup-final.psd",
            // Short names
            "cv.pdf",
            "todo.txt",
            "notes.md",
            "log.txt",
            "db.sql",
            // Very long names
            "comprehensive_annual_financial_report_with_detailed_quarterly_breakdown_and_future_projections_2025.xlsx",
            "complete_system_architecture_documentation_including_database_schema_and_api_specifications.pdf",
        ];

        // Chinese files - different domains and separators
        let chinese_files = vec![
            // Business/Office (underscore)
            "财务_季度报告.xlsx",
            "员工_绩效评估.docx",
            "市场_分析报告.pdf",
            "项目_进度跟踪.txt",
            "客户_反馈汇总.csv",
            // Academic/Education (dash)
            "数学-微积分-笔记.pdf",
            "历史-论文-草稿.docx",
            "物理-实验-数据.xlsx",
            "化学-课件-第三章.pptx",
            "英语-词汇-整理.txt",
            // Personal/Life (space in Chinese)
            "旅游 计划 2025.txt",
            "购物 清单 超市.md",
            "健身 训练 记录.xlsx",
            "读书 笔记 推荐.pdf",
            "菜谱 收藏 家常菜.docx",
            // Tech/Programming (mixed)
            "用户认证_系统设计.py",
            "数据库-连接池.js",
            "API接口 文档.md",
            "测试用例_自动化.xml",
            "部署脚本-生产环境.sh",
            // Creative (dot separator)
            "摄影作品.集锦.2025.jpg",
            "视频剪辑.项目.文件.mp4",
            "设计稿.最终版本.ai",
            "音乐创作.demo.mp3",
            // Short Chinese names
            "简历.pdf",
            "待办.txt",
            "笔记.md",
            "日志.log",
            "备忘.txt",
            // Very long Chinese names
            "公司全年财务状况详细分析报告包含各部门预算执行情况和下年度规划建议.xlsx",
            "软件系统完整技术文档包括架构设计数据库设计和接口规范说明.pdf",
            "市场调研报告涵盖用户需求分析竞争对手调研和产品定位策略建议.docx",
            // Mixed Chinese and English
            "会议纪要_Meeting_Notes.txt",
            "产品规格_Product_Spec.pdf",
            "用户手册_User_Manual.docx",
            "API文档_Documentation.md",
            "测试报告_Test_Report.xml",
            // Numbers and dates
            "2025年度总结.txt",
            "第1季度报告.xlsx",
            "版本v2.1说明.md",
            "backup_20250529.sql",
            "日报_0529.txt",
            // Special characters and formats
            "项目(重要).txt",
            "文档[草稿].docx",
            "数据{临时}.csv",
            "备份@服务器.zip",
            "配置#生产.json",
        ];

        let total_files = english_files.len() + chinese_files.len();

        for filename in english_files {
            let file_path = temp_dir.path().join(filename);
            fs::File::create(&file_path)?;
        }

        for filename in chinese_files {
            let file_path = temp_dir.path().join(filename);
            fs::File::create(&file_path)?;
        }

        Ok((temp_dir, total_files))
    }
}
