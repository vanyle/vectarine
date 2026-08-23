pub(crate) static DEFAULT_CODE: &str = "const debug = require(\"@vectarine/debug\")
const graphics = require(\"@vectarine/graphics\")
local persist = require(\"@vectarine/persist\")
local ui = require(\"@vectarine/ui\")
const vec4 = require(\"@vectarine/vec4\")
const vec = require(\"@vectarine/vec\")

-- Need help to get started?
-- Read: https://vectarineengine.com/guides/overview
-- The manual is available offline in the Help menu.

-- Print a message to the console (open the console with Ctrl+1 or from the Tools menu)
debug.print(\"Loaded.\")

-- Persist the fake timer across reloads so that it doesn't reset when you edit the code and reload the game.
local fakeLoadingTimer = persist.onReload({ value = 0.0 }, \"fakeLoadingTimer\")

local gettingStartedUi = ui.stack({ alignX = \"center\", alignY = \"center\" }, {
	ui.spacer(vec.V2(2, 2)),
	ui.column({ align = \"center\" }, {
		ui.text(
			\"Go to Tools > Resources (or press Ctrl+2) to open the resources panel\",
			{ color = vec4.WHITE, maxWidth = 1, align = \"left\" }
		),
		ui.spacer(vec.V2(0, 0.03)),
		ui.text(
			\"From there, click on game.luau to open it and get started!\",
			{ color = vec4.WHITE, maxWidth = 1, align = \"left\" }
		),
	}),
})

function Update(deltaTime: number)
	debug.fprint(\"Rendered in \", deltaTime, \"sec\")

	if
		graphics.drawSplashScreenIfNeeded({
			-- Put the resources that need to be loaded here.
		}, \"Loading\")
	then
		return
	end

	fakeLoadingTimer.value = fakeLoadingTimer.value + deltaTime
	if fakeLoadingTimer.value < 4.0 then
		graphics.drawSplashScreen(\"Pretending to load...\", fakeLoadingTimer.value / 4.0)
		return
	end

	gettingStartedUi:draw({})
end
";

pub(crate) static DEFAULT_LUAURC: &str = r#"{
	"languageMode": "strict",
	"lintErrors": false,
	"lint": {
		"FunctionUnused": false
	},
	"aliases": {
		"vectarine": "luau-api"
	}
}"#;

pub(crate) static DEFAULT_VSCODE_SETTINGS: &str = r#"{
	"luau-lsp.platform.type": "standard",
	"luau-lsp.sourcemap.enabled": false,
	"luau-lsp.sourcemap.autogenerate": false,
    "[luau]": {
		"editor.defaultFormatter": "JohnnyMorganz.stylua",
		"editor.formatOnSave": true,
	},
}"#;

pub(crate) static DEFAULT_GITIGNORE: &str = r#"luau-api
.DS_Store
*.zip
"#;
