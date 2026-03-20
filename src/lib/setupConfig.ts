import type { OpenClawConfig } from "@/types/config";

interface SetupConfigInput {
  providerName: string;
  baseUrl: string;
  apiKey: string;
  primaryModel: string;
}

export function buildSetupConfig(config: OpenClawConfig, input: SetupConfigInput): OpenClawConfig {
  const modelName = input.primaryModel.split("/").at(-1) ?? input.primaryModel;
  const existingModels = config.models?.providers?.[input.providerName]?.models;

  return {
    ...config,
    models: {
      ...(config.models ?? {}),
      providers: {
        ...(config.models?.providers ?? {}),
        [input.providerName]: {
          baseUrl: input.baseUrl,
          apiKey: input.apiKey,
          models: existingModels ?? [{ id: modelName, name: modelName }]
        }
      }
    },
    agents: {
      ...(config.agents ?? {}),
      defaults: {
        ...(config.agents?.defaults ?? {}),
        model: {
          primary: input.primaryModel,
          fallbacks: []
        }
      }
    }
  };
}
