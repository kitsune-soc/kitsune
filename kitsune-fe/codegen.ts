import type { CodegenConfig } from '@graphql-codegen/cli';

const config: CodegenConfig = {
  schema: 'http://0.0.0.0:5000/graphql',
  documents: ['src/**/*.ts', 'src/**/*.vue'],
  ignoreNoDocuments: true, // for better experience with the watcher
  generates: {
    './src/graphql/types/': {
      preset: 'client',
      config: {
        useTypeImports: true,
      },
    },
  },
};

export default config;
