package com.github.apohrebniak.hoverui.ui;

import com.github.apohrebniak.hoverui.domain.MemberEntity;
import com.vaadin.flow.component.Text;
import com.vaadin.flow.component.html.Div;
import com.vaadin.flow.component.orderedlayout.VerticalLayout;

public class MemberListItem extends VerticalLayout {

  public static MemberListItem from(MemberEntity entity) {
    MemberListItem item = new MemberListItem();
    item.foo(String.format("Hover Id: %s", entity.getId()));
    item.foo(String.format("Http address: %s:%d", entity.getHttpAddress().getIp(), entity.getHttpAddress().getPort()));
    item.foo(String.format("Hover node address: %s:%d", entity.getHoverNode().getAddr().getIp(), entity.getHoverNode().getAddr().getPort()));
    return item;
  }

  private void foo(String text) {
    add(new Div(new Text(text)));
  }
}
