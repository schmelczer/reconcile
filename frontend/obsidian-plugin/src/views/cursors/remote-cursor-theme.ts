import { EditorView } from "@codemirror/view";

const CARET_WIDTH = 2;
const DOT_RADIUS = 4;

export const remoteCursorsTheme = EditorView.baseTheme({
	".selection-caret": {
		position: "relative"
	},

	".selection-caret > *": {
		position: "absolute",
		backgroundColor: "inherit"
	},

	".selection-caret > .stick": {
		left: 0,
		top: 0,
		transform: "translateX(-50%)",
		width: `${CARET_WIDTH}px`,
		height: "100%",
		display: "block",
		borderRadius: `${CARET_WIDTH / 2}px`,
		animation: "blink-stick 1s steps(1) infinite"
	},

	"@keyframes blink-stick": {
		"0%, 100%": { opacity: 1 },
		"50%": { opacity: 0 }
	},

	".selection-caret > .dot": {
		borderRadius: "50%",
		width: `${DOT_RADIUS * 2}px`,
		height: `${DOT_RADIUS * 2}px`,
		top: `-${DOT_RADIUS}px`,
		left: `-${DOT_RADIUS}px`,
		transition: "transform .3s ease-in-out",
		transformOrigin: "bottom center",
		boxSizing: "border-box"
	},

	".selection-caret:hover > .dot": {
		transform: "scale(0)"
	},

	".selection-caret > .info": {
		top: "-1.3em",
		left: `-${CARET_WIDTH / 2}px`,
		fontSize: "0.9em",
		userSelect: "none",
		color: "white",
		padding: "0 2px",
		transition: "opacity .3s ease-in-out",
		opacity: 0,
		whiteSpace: "nowrap",
		borderRadius: "3px 3px 3px 0"
	},

	".selection-caret:hover > .info": {
		opacity: 1
	}
});
