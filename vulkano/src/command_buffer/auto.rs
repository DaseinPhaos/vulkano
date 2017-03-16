// Copyright (c) 2016 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

use std::error;
use std::sync::Arc;

use buffer::Buffer;
use command_buffer::cb;
use command_buffer::cmd;
use command_buffer::cb::AddCommand;
use command_buffer::cb::CommandBufferBuild;
use command_buffer::cb::UnsafeCommandBuffer;
use command_buffer::CommandBuffer;
use command_buffer::CommandBufferBuilder;
use command_buffer::pool::CommandPool;
use command_buffer::pool::StandardCommandPool;
use device::Device;
use device::DeviceOwned;
use device::Queue;
use image::Image;
use instance::QueueFamily;
use sync::AccessFlagBits;
use sync::PipelineStages;
use sync::GpuFuture;
use OomError;

type Cb<P> = cb::DeviceCheckLayer<cb::QueueTyCheckLayer<cb::ContextCheckLayer<cb::StateCacheLayer<cb::SubmitSyncBuilderLayer<cb::AutoPipelineBarriersLayer<cb::AbstractStorageLayer<cb::UnsafeCommandBufferBuilder<P>>>>>>>>;

pub struct AutoCommandBufferBuilder<P = Arc<StandardCommandPool>> where P: CommandPool {
    inner: Cb<P>
}

impl AutoCommandBufferBuilder<Arc<StandardCommandPool>> {
    pub fn new(device: Arc<Device>, queue_family: QueueFamily)
               -> Result<AutoCommandBufferBuilder<Arc<StandardCommandPool>>, OomError>
    {
        let pool = Device::standard_command_pool(&device, queue_family);

        let cmd = unsafe {
            let c = try!(cb::UnsafeCommandBufferBuilder::new(pool, cb::Kind::primary(), cb::Flags::SimultaneousUse /* TODO: */));
            let c = cb::AbstractStorageLayer::new(c);
            let c = cb::AutoPipelineBarriersLayer::new(c);
            let c = cb::SubmitSyncBuilderLayer::new(c);
            let c = cb::StateCacheLayer::new(c);
            let c = cb::ContextCheckLayer::new(c);
            let c = cb::QueueTyCheckLayer::new(c);
            let c = cb::DeviceCheckLayer::new(c);
            c
        };

        Ok(AutoCommandBufferBuilder {
            inner: cmd,
        })
    }
}

unsafe impl<P, O> CommandBufferBuild for AutoCommandBufferBuilder<P>
    where Cb<P>: CommandBufferBuild<Out = O>,
          P: CommandPool
{
    type Out = O;

    #[inline]
    fn build(self) -> Self::Out {
        // TODO: wrap around?
        CommandBufferBuild::build(self.inner)
    }
}

unsafe impl<P> CommandBuffer for AutoCommandBufferBuilder<P>
    where Cb<P>: CommandBuffer,
          P: CommandPool
{
    type Pool = <Cb<P> as CommandBuffer>::Pool;

    #[inline]
    fn inner(&self) -> &UnsafeCommandBuffer<Self::Pool> {
        self.inner.inner()
    }

    #[inline]
    fn submit_check(&self, future: &GpuFuture, queue: &Queue) -> Result<(), Box<error::Error>> {
        self.inner.submit_check(future, queue)
    }

    #[inline]
    fn check_buffer_access(&self, buffer: &Buffer, exclusive: bool, queue: &Queue)
                           -> Result<Option<(PipelineStages, AccessFlagBits)>, ()>
    {
        self.inner.check_buffer_access(buffer, exclusive, queue)
    }

    #[inline]
    fn check_image_access(&self, image: &Image, exclusive: bool, queue: &Queue)
                          -> Result<Option<(PipelineStages, AccessFlagBits)>, ()>
    {
        self.inner.check_image_access(image, exclusive, queue)
    }
}

unsafe impl<P> DeviceOwned for AutoCommandBufferBuilder<P>
    where Cb<P>: DeviceOwned,
          P: CommandPool
{
    #[inline]
    fn device(&self) -> &Arc<Device> {
        self.inner.device()
    }
}

unsafe impl<P> CommandBufferBuilder for AutoCommandBufferBuilder<P>
    where Cb<P>: CommandBufferBuilder,
          P: CommandPool
{
}

macro_rules! pass_through {
    (($($param:ident),*), $cmd:ty) => {
        unsafe impl<P $(, $param)*> AddCommand<$cmd> for AutoCommandBufferBuilder<P>
            where P: CommandPool,
                  Cb<P>: AddCommand<$cmd, Out = Cb<P>>
        {
            type Out = AutoCommandBufferBuilder<P>;

            #[inline]
            fn add(self, command: $cmd) -> Self::Out {
                AutoCommandBufferBuilder {
                    inner: self.inner.add(command),
                }
            }
        }
    }
}

pass_through!((Rp, F), cmd::CmdBeginRenderPass<Rp, F>);
pass_through!((S, Pl), cmd::CmdBindDescriptorSets<S, Pl>);
pass_through!((B), cmd::CmdBindIndexBuffer<B>);
pass_through!((Pl), cmd::CmdBindPipeline<Pl>);
pass_through!((V), cmd::CmdBindVertexBuffers<V>);
pass_through!((), cmd::CmdClearAttachments);
pass_through!((S, D), cmd::CmdCopyBuffer<S, D>);
pass_through!((S, D), cmd::CmdCopyBufferToImage<S, D>);
pass_through!((), cmd::CmdDrawRaw);
pass_through!((), cmd::CmdEndRenderPass);
pass_through!((C), cmd::CmdExecuteCommands<C>);
pass_through!((B), cmd::CmdFillBuffer<B>);
pass_through!((), cmd::CmdNextSubpass);
pass_through!((Pc, Pl), cmd::CmdPushConstants<Pc, Pl>);
pass_through!((), cmd::CmdSetState);
pass_through!((B, D), cmd::CmdUpdateBuffer<B, D>);
