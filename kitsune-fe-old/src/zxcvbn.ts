import type { FormKitNode } from '@formkit/core';
import type { FormKitValidationRule } from '@formkit/validation';
import type { OptionsType } from '@zxcvbn-ts/core';

import { merge } from 'lodash';

let message: string | undefined;

export const zxcvbnValidationMessage = () => {
  return message || '';
};

export const zxcvbnRule: FormKitValidationRule = merge(
  async (node: FormKitNode): Promise<boolean> => {
    const { zxcvbnOptions, zxcvbnAsync } = await import('@zxcvbn-ts/core');
    const zxcvbnCommonPackage = await import('@zxcvbn-ts/language-common');
    const enTranslations = await import(
      '@zxcvbn-ts/language-en/src/translations'
    );

    const options: OptionsType = {
      dictionary: {
        ...zxcvbnCommonPackage.dictionary,
      },
      graphs: zxcvbnCommonPackage.adjacencyGraphs,
      useLevenshteinDistance: true,
      translations: enTranslations.default,
    };

    zxcvbnOptions.setOptions(options);

    const result = await zxcvbnAsync(node.value as string);

    message = result.feedback.suggestions.join(' ');
    if (message) message += ' ';

    if (result.feedback.warning) {
      message += `Warning: ${result.feedback.warning}`;
    }

    return result.score >= 3;
  },
);
