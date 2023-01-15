import {
  Box,
  Button,
  HStack,
  VStack,
  Text,
  useToast,
  Tooltip,
} from "@chakra-ui/react";
import React, { useState } from "react";
import { useFormContext } from "react-hook-form";
import { SingleProblemSelection } from "../problems/single-problem-selection";
import { CreateRoomFormData } from "./create-room";

export const PickProblems: React.FC = () => {
  const toast = useToast();
  const form = useFormContext<CreateRoomFormData>();

  const problems = form.watch("problems");

  return (
    <VStack alignItems="start">
      {problems.length === 0 && (
        <Text fontSize="xl" color="gray.400">
          No problems yet. Add one!
        </Text>
      )}
      {problems.map(({ c, t }, i) => (
        <Box key={i} w="full">
          {t === "Single" && <SingleProblemSelection index={i} />}

          <hr />
        </Box>
      ))}
      <HStack ml="auto" w="full">
        <Button
          onClick={() =>
            form.setValue("problems", [
              ...problems,
              {
                t: "Single",
                c: {
                  id: null,
                },
              } as any,
            ])
          }
        >
          Add Problem
        </Button>

        <Tooltip label="Coming soon!">
          <Button
            disabled
            onClick={
              () => {}
              // form.setValue("problems", [
              //   ...problems,
              //   {
              //     t: "Category",
              //     c: {
              //       difficulty: 0,
              //       questions: 2,
              //       tags: ["2"],
              //     },
              //   },
              // ])
            }
          >
            Add Category
          </Button>
        </Tooltip>
      </HStack>
    </VStack>
  );
};
