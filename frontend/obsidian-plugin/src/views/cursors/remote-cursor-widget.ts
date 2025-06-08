import { AnnotationType, Annotation, RangeSet, Range } from "@codemirror/state";
import {
	ViewUpdate,
	ViewPlugin,
	Decoration,
	WidgetType
} from "@codemirror/view";

import type { PluginValue, DecorationSet, EditorView } from "@codemirror/view";

export class RemoteCursorWidget extends WidgetType {
	public constructor(
		private readonly color: string,
		private readonly name: string
	) {
		super();
	}

	public toDOM(editor: EditorView): HTMLElement {
		return editor.contentDOM.createEl(
			"span",
			{
				cls: "selection-caret",
				attr: {
					style: `background-color: ${this.color}; border-color: ${this.color}`
				}
			},
			(span) => {
				span.createEl("div", {
					cls: "stick"
				});
				span.createEl("div", {
					cls: "dot"
				});
				span.createEl("div", {
					cls: "info",
					text: this.name
				});
			}
		);
	}

	public eq(other: RemoteCursorWidget): boolean {
		return other.color === this.color && other.name === this.name;
	}
}
