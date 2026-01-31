use crate::SQL_ACTOR;
use crate::services::g_translate::languages::Language;

pub const LANGUAGES_LIMIT: u32 = 3;

pub async fn get_last_source_languages() -> anyhow::Result<Vec<Language>> {
    SQL_ACTOR.with(|connection| {
        let mut statement = connection.prepare("SELECT code, name FROM last_source_languages")?;
        let languages = statement.query_map([], |row| {
            let code: String = row.get(0)?;
            let name: String = row.get(1)?;
            Ok(Language::new(&code, &name))
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(languages)
    }).await?
}

pub async fn get_last_target_languages() -> anyhow::Result<Vec<Language>> {
    SQL_ACTOR.with(|connection| {
        let mut statement = connection.prepare("SELECT code, name FROM last_target_languages")?;
        let languages = statement.query_map([], |row| {
            let code: String = row.get(0)?;
            let name: String = row.get(1)?;
            Ok(Language::new(&code, &name))
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(languages)
    }).await?
}

pub async fn push_source_language(language: &Language) -> anyhow::Result<()> {
    SQL_ACTOR.with(clone!(
        #[strong] language,
        move |connection| {
            connection.execute(
                "INSERT INTO last_source_languages (code, name) VALUES (?, ?)",
                [language.code, language.name]
            )?;
            
            connection.execute(
                "DELETE FROM last_source_languages
                WHERE id NOT IN (
                    SELECT id FROM last_source_languages ORDER BY id DESC LIMIT ?1
                )",
                [LANGUAGES_LIMIT]
            )?;
    
            Ok(())
        }
    )).await?
}

pub async fn push_target_language(language: &Language) -> anyhow::Result<()> {
    SQL_ACTOR.with(clone!(
        #[strong] language,
        move |connection| {
            connection.execute(
                "INSERT INTO last_target_languages (code, name) VALUES (?, ?)",
                [language.code, language.name]
            )?;
            
            connection.execute(
                "DELETE FROM last_target_languages
                WHERE id NOT IN (
                    SELECT id FROM last_target_languages ORDER BY id DESC LIMIT ?1
                )",
                [LANGUAGES_LIMIT]
            )?;
    
            Ok(())
        }
    )).await?
}
