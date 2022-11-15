import { Layout } from "@/components/layout/layout";
import { CreateRoom } from "@/components/rooms/create-room";
import { axios } from "@/utils/axios";
import {
  Box,
  Button,
  Heading,
  Modal,
  ModalBody,
  ModalCloseButton,
  ModalContent,
  ModalHeader,
  ModalOverlay,
  Text,
  useDisclosure,
} from "@chakra-ui/react";
import type { NextPage } from "next";

const Home: NextPage = () => {
  const modalDisclosure = useDisclosure();

  return (
    <Layout title="Radix">
      <Heading>Rooms</Heading>

      <Button onClick={modalDisclosure.onOpen}>Create</Button>

      <Modal {...modalDisclosure} size="lg">
        <ModalOverlay />

        <ModalContent>
          <ModalHeader>Create Room</ModalHeader>
          <ModalCloseButton />
          <ModalBody>
            <CreateRoom />
          </ModalBody>
        </ModalContent>
      </Modal>
    </Layout>
  );
};

export default Home;
