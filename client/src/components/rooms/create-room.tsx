import { useZodForm } from "@/lib/hooks/use-zod-form";
import { axios } from "@/utils/axios";
import {
  Button,
  Checkbox,
  FormControl,
  FormErrorMessage,
  FormLabel,
  Input,
  VStack,
} from "@chakra-ui/react";
import { useMutation } from "@tanstack/react-query";
import { AxiosError } from "axios";
import { useCallback } from "react";
import { Controller } from "react-hook-form";
import { z } from "zod";
import { AxiosErrorMessage } from "../ui/axios-error-message";

const SPECIAL_CHARACTERS_REGEX = /[^a-zA-Z0-9_-]/g;
const formSchema = z.object({
  name: z.string(),
  public: z.boolean(),
});

type FormData = z.infer<typeof formSchema>;

export const CreateRoom: React.FC = () => {
  const form = useZodForm({
    schema: formSchema,
  });
  const errors = form.formState.errors;
  const createRoom = useMutation<unknown, AxiosError, FormData>(
    ["room"],
    (data: FormData) => {
      return axios.post("/room", data);
    }
  );

  const onSubmit = useCallback(
    async (values: FormData) => {
      try {
        await createRoom.mutateAsync(values);
        window.location.href = `/room/${values.name}`;
      } catch {}
    },
    [createRoom]
  );

  return (
    <form onSubmit={form.handleSubmit(onSubmit)}>
      <VStack gap="2">
        <FormControl>
          <FormLabel htmlFor="name">Room Name</FormLabel>
          <Controller
            name="name"
            control={form.control}
            render={({ field: { value, onChange } }) => (
              <Input
                id="name"
                autoComplete="off"
                value={value || ""}
                onChange={(e) => {
                  onChange(
                    e.target.value.replace(SPECIAL_CHARACTERS_REGEX, "-")
                  );
                }}
              />
            )}
          />
          <FormErrorMessage>{errors.name?.message}</FormErrorMessage>
        </FormControl>

        <FormControl>
          <Checkbox size="lg" {...form.register("public")} autoComplete="off">
            Public
          </Checkbox>
          <FormErrorMessage>{errors.public?.message}</FormErrorMessage>
        </FormControl>
      </VStack>

      <Button my="4" w="100%" type="submit" isLoading={createRoom.isLoading}>
        Create
      </Button>

      <AxiosErrorMessage error={createRoom.error} />
    </form>
  );
};
