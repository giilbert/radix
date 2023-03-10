import {
  Button,
  ButtonGroup,
  Drawer,
  DrawerContent,
  DrawerHeader,
  DrawerOverlay,
  Heading,
  HStack,
  IconButton,
  useDisclosure,
} from "@chakra-ui/react";
import { FiInfo, FiMessageSquare, FiX } from "react-icons/fi";
import { MdOutlineDescription } from "react-icons/md";
import { Chat } from "./chat";
import { useRoomData } from "./room-provider";
import { SidePanel } from "./side-panel";

export const TopBar: React.FC = () => {
  const { problems, currentProblemIndex } = useRoomData();
  const descriptionDisclosure = useDisclosure();
  const chatDisclosure = useDisclosure();

  return (
    <>
      <HStack h="3rem" p="1">
        {problems && (
          <IconButton
            icon={<MdOutlineDescription />}
            aria-label="Problem description"
            onClick={descriptionDisclosure.onOpen}
          />
        )}

        {problems && (
          <Heading size="md" ml="auto" w="full" textAlign="center">
            Problem {currentProblemIndex + 1}:{" "}
            {problems?.at(currentProblemIndex)?.title}
          </Heading>
        )}

        <IconButton
          icon={<FiMessageSquare />}
          aria-label="Chat"
          ml="auto !important"
          onClick={chatDisclosure.onOpen}
        />
      </HStack>

      <Drawer placement="left" {...descriptionDisclosure} size="lg">
        <DrawerOverlay />

        <DrawerContent bgColor="#161B25">
          <IconButton
            icon={<FiX />}
            aria-label="Close drawer"
            size="md"
            colorScheme="red"
            position="absolute"
            right="1"
            top="1"
            onClick={descriptionDisclosure.onClose}
          />
          <SidePanel />
        </DrawerContent>
      </Drawer>

      <Drawer placement="right" {...chatDisclosure} size="lg">
        <DrawerOverlay />

        <DrawerContent bgColor="#161B25">
          <IconButton
            icon={<FiX />}
            aria-label="Close drawer"
            size="md"
            colorScheme="red"
            position="absolute"
            right="1"
            top="1"
            onClick={chatDisclosure.onClose}
          />
          <Chat />
        </DrawerContent>
      </Drawer>
    </>
  );
};
