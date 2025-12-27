import {
    InnerToken,
    Pre,
    type AnnotationHandler,
    type HighlightedCode,
} from "codehike/code";
import {
    Selectable,
    Selection,
    SelectionProvider,
} from "codehike/utils/selection";
import { SmoothPre } from "./smooth-pre";

interface Step {
    title: string;
    description: string;
    codeDark: HighlightedCode;
    codeLight: HighlightedCode;
}

const tokenTransitions: AnnotationHandler = {
    name: "token-transitions",
    PreWithRef: SmoothPre,
    Token: (props) => (
        <InnerToken merge={props} style={{ display: "inline-block" }} />
    ),
};

export default function ScrollycodingUI({ steps }: { steps: Step[] }) {
    return (
        <SelectionProvider className="scrollycoding not-content">
            <div className="scrollycoding-steps">
                {steps.map((step, i) => (
                    <Selectable
                        key={i}
                        index={i}
                        selectOn={["click", "scroll"]}
                        className="scrollycoding-step"
                    >
                        <h2 className="scrollycoding-step-title">
                            {step.title}
                        </h2>
                        {step.description && <p>{step.description}</p>}
                    </Selectable>
                ))}
            </div>
            <div className="scrollycoding-sticker">
                <div className="ch-theme-dark">
                    <Selection
                        from={steps.map((step) => (
                            <Pre
                                code={step.codeDark}
                                handlers={[tokenTransitions]}
                            />
                        ))}
                    />
                </div>
                <div className="ch-theme-light">
                    <Selection
                        from={steps.map((step) => (
                            <Pre
                                code={step.codeLight}
                                handlers={[tokenTransitions]}
                            />
                        ))}
                    />
                </div>
            </div>
        </SelectionProvider>
    );
}
