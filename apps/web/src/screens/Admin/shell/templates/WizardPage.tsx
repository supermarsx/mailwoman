// Wizard page template (t32-cfg UX3): new domain, import, policy setup.
//
// The shell's `Wizard` as a page: numbered steps, validation per step before
// Next, Back keeps what was entered, a review step that lists every value, and
// nothing saved before Save on the review step. The wizard's heading is the page
// heading; the page description opens the first step. While input is unsaved the
// dirty-state guard asks before leaving (the Wizard registers it).

import { type JSX } from 'solid-js';
import { Wizard, type WizardProps, type WizardStep } from '../Wizard.tsx';
import * as ux from '../ux.css.ts';

export interface WizardPageProps extends WizardProps {
  description?: string;
  testId?: string;
}

export function WizardPage(props: WizardPageProps): JSX.Element {
  const steps = (): readonly WizardStep[] =>
    props.steps.map((s, i) =>
      i === 0 && props.description
        ? {
            ...s,
            content: () => (
              <>
                <p class={ux.pageDescription}>{props.description}</p>
                {s.content()}
              </>
            ),
          }
        : s,
    );
  return (
    <div class={ux.page} data-testid={props.testId}>
      <Wizard
        title={props.title}
        steps={steps()}
        review={props.review}
        onSave={props.onSave}
        {...(props.onCancel ? { onCancel: props.onCancel } : {})}
        {...(props.saveLabel ? { saveLabel: props.saveLabel } : {})}
        {...(props.dirty ? { dirty: props.dirty } : {})}
      />
    </div>
  );
}
